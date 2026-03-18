use std::marker::PhantomData;
use std::sync::{Arc};
use winit::window::Window;
use super::render_context::RenderContext;
use crate::camera::Camera;
use crate::core::{SwapchainError, VulkanError, VulkanResult};
use crate::frame_graph::FrameGraph;
use crate::resources::*;

/// A lightweight, performance-oriented abstraction over the Vulkan
/// API
///
/// Prioritizes performance over safety in certain cases - some
/// operations are marked `unsafe` and require the caller to uphold
/// invariants that cannot be enforced at compile time
///
/// # Asset Creation
///
/// Mesh, material, and transform creation is single-threaded from the
/// caller's perspective. Internally, some Vulkan object creation may
/// be parallelized, but this is an implementation detail and is not
/// exposed through the public API
///
/// # Stability
///
/// This abstraction is currently experimental. The public API may
/// change between versions without prior notice
///
/// # Example
///
/// ```ignore
/// let world = WorldRenderer::new(&window)?;
/// world.draw_frame(|graph| { ... });
/// ```
pub struct WorldRenderer {
    /// Handles pass scheduling, dependency resolution, and automatic resource barriers
    graph: FrameGraph,
    /// Contains resources for rendering and caches them creation
    resources: Arc<Resources>,
    /// Main Context for creation and some resources
    ctx: Arc<RenderContext>,
    /// No Send and Sync!
    /// In the future, this may be circumvented using channels
    _marker: PhantomData<*mut ()>,
}

impl WorldRenderer {
    /// Creates a new [`WorldRenderer`]!
    ///
    /// # Panics!
    ///
    /// If Vulkan is not found on this device or the device does not
    /// support core formats or features.
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

    /// Allocates a new GPU resource of type `T` and returns a handle to it
    ///
    /// The handle [`Res<T>`] is lightweight and can be cloned freely   
    /// 
    /// The resource lives until at least one of its handles is alive
    ///
    /// # Errors
    ///
    /// Returns [`VulkanError`] if a resource creation error has occurred or the creation parameters are not valid
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mesh: Res<Mesh> = world.create::<Mesh>(MeshDesc { ... })?;
    /// ```
    pub fn create<T: Create>(&mut self, desc: T::Desc<'_>) -> VulkanResult<Res<T>> {
        T::create(&self.resources, desc)
    }

    /// Returns a read guard to the resource identified by `res`
    ///
    /// Internally wraps [`parking_lot::RwLockReadGuard`] — the lock is held
    /// until the returned [`Ref`] is dropped. Multiple read guards
    /// to the same resource can coexist, but acquiring [`RefMut`] while
    /// any [`Ref`] is alive **will deadlock**.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let transform = world.get(&handle);
    /// println!("{:?}", mesh.scale);
    /// // lock released here
    /// ```
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

    /// Returns a write guard to the [`Camera`]
    ///
    /// Internally wraps [`parking_lot::RwLockWriteGuard`] — the lock is held
    /// until the returned [`RefMut`] is dropped.
    ///
    /// # Deadlocks
    ///
    /// Calling this while any other [`Ref`] or [`RefMut`] to the camera is alive
    /// **will deadlock**. Drop all existing guards before acquiring a new one.
    ///
    /// ```ignore
    /// // OK
    /// {
    ///     let mut camera = world.camera_mut();
    ///     let view = camera.view();
    /// } // guard dropped here
    /// let camera = world.camera(); // safe
    ///
    /// // DEADLOCK
    /// let camera = world.camera();
    /// let camera_mut = world.camera_mut(); // hangs forever
    /// ```
    pub fn camera_mut(&mut self) -> RefMut<'_, Camera> {
        //RefMut(self.resources.camera.write())
        todo!()
    }

    /// Returns a read guard to the [`Camera`]
    ///
    /// Internally wraps [`parking_lot::RwLockReadGuard`] — the lock is held
    /// until the returned [`Ref`] is dropped.
    ///
    /// Multiple read guards can coexist, but acquiring [`RefMut`] while
    /// any [`Ref`] is alive **will deadlock**.
    ///
    /// ```ignore
    /// // OK — multiple readers at once
    /// let camera1 = world.camera();
    /// let camera2 = world.camera();
    ///
    /// // DEADLOCK — write while read is alive
    /// let camera = world.camera();
    /// let camera_mut = world.camera_mut(); // hangs forever
    /// ```
    pub fn camera(&self) -> Ref<'_, Camera> {
        //Ref(self.resources.camera.read())
        todo!()
    }

    /// Resizes the window
    ///
    /// # Panics!
    ///
    /// - If the GPU has stopped responding
    /// - If there is not enough memory
    /// - If new resources cannot be created or old ones deleted
    /// - If an unexpected error occurs
    pub fn resize(&mut self, width: u32, height: u32) -> VulkanResult<()> {
        profiling::scope!("WorldRenderer::resize");

        // skip
        if width == 0 || height == 0 {
            return Ok(());
        }

        self.ctx.resize(width, height)?;

        Ok(())
    }

    /// Draw a frame or skip a frame if some resources need to be create
    /// # Panics!
    ///
    /// - If the GPU has stopped responding
    /// - If new resources cannot be created or old ones deleted
    /// - If an unexpected error occurs
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

        // RenderContext implements Drop itself and can be removed later.
    }
}
