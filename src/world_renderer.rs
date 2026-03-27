use std::marker::PhantomData;
use std::sync::{Arc};
use winit::window::Window;
use super::render_context::RenderContext;
use crate::camera::Camera;
use crate::core::{SwapchainError, VulkanError, VulkanResult};
use crate::frame_graph::FrameGraph;
use crate::resources::*;
/// Lightweight Vulkan abstraction focused on frame-graph–driven frame submission.
///
/// Wraps a [`FrameGraph`], a shared resource pool, and a [`RenderContext`] into a
/// single entry point. The graph handles pass ordering, barrier insertion, and
/// transient resource lifetimes automatically; callers declare *what* needs to
/// happen, not *when* synchronisation must occur.
///
/// ## Threading
///
/// `WorldRenderer` is intentionally `!Send + !Sync`. All calls must originate
/// from the thread that owns the Vulkan context. Resource creation is
/// single-threaded from the caller's perspective; any internal parallelism is
/// an implementation detail.
///
/// ## Stability
///
/// Experimental. The public API may change between minor versions without a
/// deprecation period.
///
/// ## Example
///
/// ```ignore
/// let mut world = WorldRenderer::new(&window)?;
///
/// world.draw_frame(|graph| {
///     graph.add_pass(...);
/// })?;
/// ```
pub struct WorldRenderer {
    /// Handles pass scheduling, dependency resolution, and automatic resource barriers
    graph: FrameGraph,
    /// Contains resources for rendering and caches them creation
    resources: Arc<Resources>,
    /// Main Context for creation and some resources
    ctx: Arc<RenderContext>,
    /// No Send and Sync!
    _marker: PhantomData<*mut ()>,
}

impl WorldRenderer {
    /// Initialises Vulkan and creates the renderer.
    ///
    /// Selects a physical device, creates a logical device and swapchain,
    /// and allocates the initial resource pools. The window handle must
    /// remain valid for the lifetime of the returned renderer.
    ///
    /// # Panics
    ///
    /// Panics if no Vulkan-capable device is found or the device does not
    /// support the formats and features required by this renderer. Prefer
    /// checking hardware support at a higher level rather than catching
    /// panics here.
    pub fn new(window: &Window) -> VulkanResult<WorldRenderer> {
        let ctx = RenderContext::new(window)?;
        let resources = Resources::new(&ctx)?;
        let graph = FrameGraph::new()?;

        Ok(WorldRenderer {
            resources,
            graph,
            ctx,
            _marker: PhantomData,
        })
    }

    /// Allocates a GPU resource and returns a reference-counted handle to it.
    ///
    /// `Res<T>` is a thin wrapper around an index into the resource pool —
    /// cheap to clone, copy, and compare. The underlying allocation is kept
    /// alive as long as at least one handle exists; dropping the last handle
    /// queues the resource for deferred destruction at the end of the frame.
    ///
    /// # Errors
    ///
    /// Returns [`VulkanError`] if the driver rejects the creation parameters
    /// or if the allocator runs out of suitable memory.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mesh: Res<Mesh> = world.create::<Mesh>(MeshDesc {
    ///     vertices: &verts,
    ///     indices:  &indices,
    /// })?;
    /// ```
    pub fn create<T: Create>(&mut self, desc: T::Desc<'_>) -> VulkanResult<Res<T>> {
        T::create(&self.ctx, &self.resources, desc)
    }

    /// Acquires a shared read lock on the resource.
    ///
    /// The lock is held until the returned [`Ref`] is dropped; holding it
    /// across a `draw_frame` call is almost certainly a deadlock. Prefer
    /// short-lived guards scoped to the lines that actually need the data.
    ///
    /// Panics (in debug) or deadlocks (in release) if a [`RefMut`] to the
    /// same resource is already held on this thread.
    pub fn get<T: Get>(&self, res: &Res<T>) -> Ref<'_, T> {
        T::get(&self.resources, res)
    }

    /// Returns a write guard to the resource identified by `res`
    ///
    /// Internally wraps [`parking_lot::RwLockWriteGuard`] — the lock is held
    /// until the returned [`RefMut`] is dropped. Acquiring this while
    /// any other [`Ref`] or [`RefMut`] to the same resource is alive
    /// **will deadlock**.
    ///
    /// # Example
    ///
    /// ```ignore
    /// {
    ///     let mut transform = world.get_mut(&handle);
    ///     transform.pos[0] -= 0.5;
    /// } // lock released here — safe to call get() or get_mut() again
    /// ```
    pub fn get_mut<T: GetMut>(&mut self, res: &Res<T>) -> RefMut<'_, T> {
        T::get_mut(&self.resources, res)
    }

    /// Acquires an exclusive write lock on the camera.
    ///
    /// Equivalent to `get_mut` for the implicit camera resource. Deadlock
    /// rules are the same: drop all live camera guards before calling this.
    pub fn camera_mut(&mut self) -> RefMut<'_, Camera> {
        RefMut(self.resources.camera.write())
    }

    /// Acquires a shared read lock on the camera.
    ///
    /// Multiple read guards may coexist. Do not call `camera_mut` while any
    /// read guard is alive — there is no runtime detection, it will deadlock.
    pub fn camera(&self) -> Ref<'_, Camera> {
        Ref(self.resources.camera.read())
    }

    /// Recreates extent-dependent resources after a window resize.
    ///
    /// Waits for the GPU to drain before destroying the old swapchain and
    /// surface-sized attachments. Zero-area extents (minimised window) are
    /// silently ignored — the swapchain is left intact and the next
    /// non-zero resize will rebuild it.
    ///
    /// # Panics
    ///
    /// Panics on device loss, allocation failure, or any Vulkan error that
    /// leaves the renderer in an unrecoverable state.
    pub fn resize(&mut self, width: u32, height: u32) -> VulkanResult<()> {
        profiling::scope!("WorldRenderer::resize");

        // skip
        if width == 0 || height == 0 {
            return Ok(());
        }

        self.ctx.resize(width, height)?;

        Ok(())
    }

    /// Compiles and submits the frame graph built by `setup`.
    ///
    /// Calls `setup` to populate the graph, then compiles the pass DAG
    /// (barrier insertion, resource aliasing, queue assignment), and finally
    /// submits command buffers to the GPU. The return value of `setup` is
    /// forwarded to the caller unchanged.
    ///
    /// If the swapchain reports `OUT_OF_DATE`, `resize` is called
    /// automatically and the frame is skipped. `SUBOPTIMAL` is silently
    /// ignored for Android compatibility (see inline comment for rationale).
    ///
    /// # Panics
    ///
    /// Panics on device loss or any Vulkan error that cannot be recovered
    /// from transparently (e.g. failed swapchain recreation).
    pub fn draw_frame<R, F: FnOnce(&mut FrameGraph) -> R>(&mut self, setup: F) -> VulkanResult<R> {
        profiling::scope!("WorldRenderer::draw_frame");

        // Setup graph
        let result = setup(&mut self.graph);

        // Compile Graph
        self.graph.compile(&self.ctx, &self.resources)?;

        // Execute Graph
        if let Err(err) = self.graph.execute(&self.ctx, &self.resources) {
            if let VulkanError::Swapchain(err) = err {
                match err {
                    SwapchainError::SwapchainOutOfDateKhr => {
                        let extent = self.ctx.resolution();
                        self.resize(extent.width, extent.height)?;
                    },
                    SwapchainError::SwapchainCreationFailed(err) => {
                        return Err(VulkanError::Swapchain(SwapchainError::SwapchainCreationFailed(err)));
                    },
                    SwapchainError::SwapchainSubOptimal => {
                        /*
                            FIX:
                            For now, we're ignoring this error on Android
                            Currently, the swapchain always has one orientation: IDENTITY
                        */
                    },
                }
            } else {
                return Err(err);
            }
        }
        Ok(result)
    }
}

/// Destroying objects in the correct order
impl Drop for WorldRenderer {
    fn drop(&mut self) {
        let device = &self.ctx.device;
        // Wait all gpu work before destroy resources
        unsafe { device.device_wait_idle().expect("Error device wait idle") };

        if Arc::strong_count(&self.resources) > 1 {
            panic!("Resources has another clone this is an architectural error");
        }

        self.graph.destroy(device);
        self.resources.destroy(device);

        if Arc::strong_count(&self.ctx) > 1 {
            panic!("Render Context has another clone this is an architectural error");
        }

        // Render Context drop here
    }
}
