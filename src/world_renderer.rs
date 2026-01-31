use std::marker::PhantomData;
use std::mem::ManuallyDrop;

use bytemuck::{Pod, Zeroable};
use puffin::{profile_scope, GlobalProfiler};

use super::render_context::RenderContext;
use crate::bindless::{Bindless, BindlessBuilder};
use crate::bvh::Bvh;
use crate::camera::Camera;
use crate::core::{
    AttributeDescriptions, BindingDescriptions, SwapchainError, VulkanError, VulkanResult,
};
use crate::render_graph::RenderGraph;
use crate::resource_manager::{
    Material, MaterialHandle, MeshHandle, Renderable, RenderableHandle, ResourceManager, Transform,
    TransformHandle,
};

/// A lightweight abstraction for rendering using the Vulkan API
pub struct WorldRenderer {
    /// Resource Manager
    pub(crate) resources: ResourceManager,
    /// Bindless
    pub(crate) bindless: Bindless,
    /// Render Graph
    /// The rendering graph automatically creates the necessary resources and
    /// performs topological sorting.
    pub(crate) graph: RenderGraph,
    /// Main Camera
    pub(crate) camera: Camera,
    /// RenderContext
    /// The rendering context provides an entry point for creating and deleting
    /// Vulkan objects
    pub(crate) ctx: ManuallyDrop<RenderContext>,
    /// Bounding Volume Hierarchy
    /// It is automatically built for meshes if ray-tracing is not natively
    /// supported
    pub(crate) bvh: Bvh,
    /// This structure not Send and Sync!
    pub(crate) _marker: PhantomData<*mut ()>,
}

impl WorldRenderer {
    /// Create new WorldRenderer!
    /// # Panic!
    ///
    /// If Vulkan is not found on this device or the device does not support
    /// core formats or features.
    pub fn new(window: &winit::window::Window) -> VulkanResult<WorldRenderer> {
        // Automatically enable required extensions/layers/features depending on the
        // platform
        let ctx = RenderContext::new(window)?;
        let camera = Camera::new(&ctx.device)?;
        let bindless = BindlessBuilder::new(&ctx.device).build()?;
        let graph = RenderGraph::new(&ctx, &bindless)?;
        let resources = ResourceManager::new(&ctx)?;
        let bvh = Bvh::new();

        Ok(WorldRenderer {
            bvh,
            bindless,
            resources,
            camera,
            graph,
            ctx: ManuallyDrop::new(ctx),
            _marker: PhantomData,
        })
    }

    /// Create immutable mesh at the moment and block current thread
    /// # Panic!
    ///
    /// If not enought memory for new allocation
    pub fn create_mesh<T>(
        &mut self,
        vertices: &[T],
        indices: Option<&[u32]>,
    ) -> VulkanResult<MeshHandle>
    where
        T: AttributeDescriptions + BindingDescriptions + Pod + Zeroable,
    {
        self.resources
            .create_static_mesh_immediately(&self.ctx, vertices, indices)
    }

    /// Create new Material
    pub fn create_material(&mut self, material: Material) -> VulkanResult<MaterialHandle> {
        Ok(self.resources.create_material(material))
    }

    /// Create Transform for Mesh
    pub fn create_transform(&mut self, transform: Transform) -> VulkanResult<TransformHandle> {
        Ok(self.resources.create_transform(transform))
    }

    /// Create Renderable Object
    pub fn create_renderable(&mut self, renderable: Renderable) -> RenderableHandle {
        self.resources.create_renderable(renderable)
    }

    /// Resize the window
    /// # Panic!
    /// If the GPU has stopped responding
    ///
    /// If not enough memory
    ///
    /// If you were unable to create new resources or delete old ones
    pub fn resize(&mut self, width: u32, height: u32) -> VulkanResult<()> {
        profile_scope!("WorldRenderer::resize");

        // skip
        if width == 0 || height == 0 {
            return Ok(());
        }

        self.ctx.resize(width, height)?;
        self.graph.recreate_transient_resources();

        Ok(())
    }

    /// Get mut ref to RenderGraph
    pub fn graph_mut(&mut self) -> &mut RenderGraph {
        &mut self.graph
    }

    /// Draw a frame or skip a frame if some resources need to be create
    /// # Panic!
    ///
    /// If the GPU has stopped responding
    pub fn draw_frame(&mut self) -> VulkanResult<()> {
        profile_scope!("WorldRenderer::draw_frame");
        GlobalProfiler::lock().new_frame();

        match self.graph.execute(&mut self.ctx) {
            Ok(_) => {},
            Err(VulkanError::Swapchain(err)) => match err {
                SwapchainError::SwapchainOutOfDateKhr | SwapchainError::SwapchainSubOptimal => {
                    let extent = self.ctx.window.resolution;
                    self.resize(extent.width, extent.height)?;
                },
                _ => {
                    log::error!("Error swapchain: {:?}", err);
                },
            },
            Err(err) => {
                log::error!("Error draw frame: {:?}", err);
            },
        }

        Ok(())
    }
}

/// Destroying objects in the correct order
impl Drop for WorldRenderer {
    fn drop(&mut self) {
        let device = &self.ctx.device;
        let wait = unsafe { device.device_wait_idle() };

        if let Err(err) = wait {
            log::error!(
                "Error wait idle: {:?}. Vulkan resources were not destroyed correctly",
                err
            );
        } else {
            // self.resources.destroy();
            self.camera.destroy(&device);
            self.graph.destroy(&device);
            // self.bindless.destroy(&device);

            // Safety: All low-level vulkan resources are destroyed before that
            unsafe { ManuallyDrop::drop(&mut self.ctx) };
        }
    }
}
