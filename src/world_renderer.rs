use std::marker::PhantomData;
use std::sync::Arc;

use winit::window::Window;

use super::render_context::RenderContext;
use crate::camera::Camera;
use crate::core::{SwapchainError, VulkanError, VulkanResult};
use crate::frame_graph::FrameGraph;
use crate::resources::*;
/// Lightweight abstraction for rendering using Vulkan API
///
/// The Vulkan API is known for its verbosity, and my abstraction tries to solve
/// this by leaving pleasant advantages in the form of huge performance and
/// cross-platform compatibility
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
    /// FrameGraph for automatic creation of barriers between resources and
    /// correct order of execution of passes
    graph: FrameGraph,
    /// Thread safe to use
    ///
    /// Resources contain all previously created data and cache the creation of
    /// new ones
    resources: Arc<Resources>,
    /// The Render Context provides the main initialized vulkan structures and
    /// implements its own proper destruction
    ctx: Arc<RenderContext>,
    /// No Send and Sync!
    ///
    /// Although Vulkan technically supports multi-threaded resource creation,
    /// the abstraction currently only provides a single-threaded API for
    /// resource creation
    _marker: PhantomData<*mut ()>,
}

impl WorldRenderer {
    /// # Create new WorldRenderer
    /// - Automatic selection of the appropriate GPU
    /// - Checking available extensions and selecting them
    ///
    /// # Panics
    /// - if not supported vulkan api on this device
    /// - if the gpu does not support the required extensions
    /// - if the device does not support the required formats
    pub fn new(window: &Window) -> VulkanResult<WorldRenderer> {
        let ctx = RenderContext::new(window)?;
        let resources = Resources::new(&ctx)?;
        let graph = FrameGraph::new(&ctx)?;

        Ok(WorldRenderer {
            resources,
            graph,
            ctx,
            _marker: PhantomData,
        })
    }

    /// Create new resource
    ///
    /// [`Res<T>`] is a smart handle for deferred resource deletion
    ///
    /// The resource lifetime ends with the last drop + 5-10 frames after
    ///
    /// # Panics!
    /// - if the resource creation parameters are not valid
    /// - if the new GPU memory allocation returned an error
    /// - if device lost (Driver Bug)
    ///
    /// # Example
    ///
    /// ```ignore
    /// 
    /// let vertices = vec![
    ///     Vertex { pos: [ 0.0,  0.5, 0.0], color: [1.0, 0.0, 0.0] },
    ///     Vertex { pos: [-0.5, -0.5, 0.0], color: [0.0, 1.0, 0.0] },
    ///     Vertex { pos: [ 0.5, -0.5, 0.0], color: [0.0, 0.0, 1.0] }
    /// ];
    /// // Ok
    /// let mesh: Res<Mesh> = world.create::<Mesh>(MeshDesc::new(&vertices))?;
    ///
    /// let indices: u32 = vec![];
    /// // Error: Indices must be not empty!
    /// let mesh: Res<Mesh> = world.create::<Mesh>(MeshDesc::new(&vertices).with_indices(&indices))?;
    /// ```
    pub fn create<T: Create>(&self, desc: T::Desc<'_>) -> VulkanResult<Res<T>> {
        T::create(&self.ctx, &self.resources, desc)
    }

    /// Acquires a shared read lock on the resource [`Ref<'_, T>`]
    ///
    /// There may be many readers, but only one writer in one area
    ///
    /// # Example
    /// ```ignore
    /// 
    /// let transform = world.create::<Transform>(TransformDesc::identety())?;
    /// // Ok
    /// let scale = world.get(&transform).scale;
    ///
    /// {
    ///   let transform: Res<Transform> = world.create::<Transform>(TransformDesc::identety())?;
    ///   let transform1 = world.get(&transform);
    ///   // Error: Deadlock
    ///   let transform2 = world.get_mut(&transform);
    /// }
    /// ```
    pub fn get<T: Get>(&self, res: &Res<T>) -> Ref<'_, T> {
        T::get(&self.resources, res)
    }

    /// Acquires a shared write lock on the resource [`RefMut<'_, T>`]
    ///
    /// Only one writer in scope
    ///
    /// # Example
    /// ```ignore
    /// 
    /// let transform: Res<Transform> = world.create::<Transform>(TransformDesc::identity())?;
    /// // Ok
    /// world.get_mut(&transform).scale[0] *= 0.2;
    ///
    /// {
    ///   let transform: Res<Transform> = world.create::<Transform>(TransformDesc::identity())?;
    ///   let transform1 = world.get_mut(&transform);
    ///   // Error: Deadlock! transform1 is alive!
    ///   let transform2 = world.get_mut(&transform);
    /// }
    /// ```
    pub fn get_mut<T: GetMut>(&mut self, res: &Res<T>) -> RefMut<'_, T> {
        T::get_mut(&self.resources, res)
    }

    /// Acquires an exclusive write lock on the camera [`RefMut<'_, Camera>`]
    ///
    /// To avoid blocking, do not store the result in a variable
    ///
    /// There may be many readers, but only one writer in one area
    pub fn camera_mut(&mut self) -> RefMut<'_, Camera> {
        RefMut(self.resources.camera.write())
    }

    /// Acquires a shared read lock on the camera [`Ref<'_, Camera>`]
    ///
    /// To avoid blocking, do not store the result in a variable
    ///
    /// There may be many readers, but only one writer in one area
    pub fn camera(&self) -> Ref<'_, Camera> {
        Ref(self.resources.camera.read())
    }

    /// Re-creating the main window
    ///
    /// # Panics
    /// - if an error occurred while creating new resources
    /// - if device lost (Driver Bug)
    pub fn resize(&mut self, width: u32, height: u32) -> VulkanResult<()> {
        profiling::scope!("WorldRenderer::resize");

        // skip
        if width == 0 || height == 0 {
            return Ok(());
        }

        self.ctx.resize(width, height)?;

        Ok(())
    }

    /// Accepts a closure in which the entire frame creation cycle must be
    /// described
    ///
    /// # Example
    /// ```ignore
    /// let mut world = WorldRenderer::new(&window)?;
    ///
    /// let simple_pipeline = world.create::<RasterPipeline>(
    ///     RasterPipelineDesc::new()
    ///         .vertex_shader("../first_shader.spv")
    ///         .fragment_shader("../second_shader.spv")
    /// )?;
    ///
    /// world.draw_frame(|graph| {
    ///     graph.add_pass(
    ///         PresentPass::new("Final Pass").execute(|ctx| unsafe {
    ///             ctx.bind_pipeline(simple_pipeline)
    ///             ctx.draw(3)
    ///         });
    ///     );
    /// })?;
    ///     
    /// ```
    ///
    /// # Panics
    /// - if if an error occurred while recreating the window
    /// - if the pass data is not valid
    /// - if an error occurred while creating new resources
    /// - if device lost (Driver Bug)
    pub fn draw_frame<F: FnOnce(&mut FrameGraph)>(&mut self, callback: F) -> VulkanResult<()> {
        profiling::scope!("WorldRenderer::draw_frame");

        // Setup graph
        callback(&mut self.graph);

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
                    SwapchainError::SwapchainSubOptimal => {
                        // FIX ME:
                        // For now, we're ignoring this error on Android
                        // Currently, the swapchain always has one orientation:
                        // IDENTITY
                    },
                    err => return Err(VulkanError::Swapchain(err)),
                }
            } else {
                return Err(err);
            }
        }
        Ok(())
    }
}

/// Destroying objects in the correct order
impl Drop for WorldRenderer {
    fn drop(&mut self) {
        let device = &self.ctx.device;
        // Wait all gpu work before destroy resources
        unsafe { device.device_wait_idle().expect("Error device wait idle") };

        if Arc::strong_count(&self.resources) > 1 {
            panic!("Resources has another clone!");
        }

        self.graph.destroy(device);
        self.resources.destroy(device);

        if Arc::strong_count(&self.ctx) > 1 {
            panic!("Render Context has another clone!");
        }

        // Render Context drop here
    }
}
