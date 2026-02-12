use std::marker::PhantomData;
use std::mem::ManuallyDrop;

use ash::vk;
use bytemuck::{Pod, Zeroable};
use puffin::{profile_scope, GlobalProfiler};
use winit::window::Window;

use super::render_context::RenderContext;
use crate::bindless::{Bindless, BindlessBuilder};
use crate::camera::{Camera, CameraData};
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
    ///
    /// Contains resources for rendering and caches them
    pub(crate) resources: ResourceManager,
    /// Bindless
    ///
    /// Natively supported on Windows and Linux; on other platforms, falls back to arrays
    pub(crate) bindless: Bindless,
    /// Render Graph
    ///
    /// The rendering graph automatically creates the necessary resources and
    /// performs topological sorting
    pub(crate) graph: RenderGraph,
    /// Main Camera
    ///
    /// Provides data and methods for changing it
    pub(crate) camera: Camera,
    /// Render Context
    ///
    /// The rendering context provides an entry point for creating and deleting resources
    pub(crate) ctx: ManuallyDrop<RenderContext>,
    /// This structure not Send and Sync!
    pub(crate) _marker: PhantomData<*mut ()>,
}

impl WorldRenderer {
    /// Creates a new [`WorldRenderer`].
    ///
    /// # Panics
    ///
    /// If Vulkan is not found on this device or the device does not support
    /// core formats or features.
    pub fn new(window: &Window) -> VulkanResult<WorldRenderer> {
        // Automatically enable required extensions/layers/features depending on the
        // platform
        let ctx = RenderContext::new(window)?;
        let camera = Camera::new(&ctx.device)?;
        let resources = ResourceManager::new(&ctx.device)?;

        let mut bindless = BindlessBuilder::new(&ctx.device)
            .with(
                0,
                1,
                vk::DescriptorType::UNIFORM_BUFFER,
                vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
            )
            .with(
                1, 
                10000, 
                vk::DescriptorType::STORAGE_BUFFER, 
                vk::ShaderStageFlags::VERTEX
            )
            .build()?;

        bindless.update_buffer_set(
            &ctx.device,
            0,
            vk::DescriptorType::UNIFORM_BUFFER,
            camera.buffer.raw,
            0,
            size_of::<CameraData>() as u64,
        );

        bindless.update_buffer_set(
            &ctx.device,
            1,
            vk::DescriptorType::STORAGE_BUFFER,
            resources.assets.transform.buffer.raw,
            0,
            size_of::<Transform>() as u64 * 10000,
        );

        let graph = RenderGraph::new(&ctx, &bindless)?;

        Ok(WorldRenderer {
            bindless,
            resources,
            camera,
            graph,
            ctx: ManuallyDrop::new(ctx),
            _marker: PhantomData,
        })
    }

    /// Gets a reference to the [`Camera`]
    pub fn camera_mut(&mut self) -> &mut Camera {
        &mut self.camera
    }

    /// Gets a reference to the [`Camera`]
    pub fn camera(&self) -> &Camera {
        &self.camera
    }

    /// Creates an immutable mesh
    ///
    /// # Panics
    ///
    /// if there is not enough memory for a new allocation
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
    /// # Panic!
    /// 
    /// if not success allocate descriptor set
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

    /// Resizes the window
    ///
    /// # Panics
    ///
    /// - If the GPU has stopped responding
    /// - If there is not enough memory
    /// - If new resources cannot be created or old ones deleted
    pub fn resize(&mut self, width: u32, height: u32) -> VulkanResult<()> {
        profile_scope!("WorldRenderer::resize");

        // skip
        if width == 0 || height == 0 {
            return Ok(());
        }

        self.ctx.resize(width, height)?;

        Ok(())
    }

    /// Get mut ref to [`RenderGraph`]
    pub fn graph_mut(&mut self) -> &mut RenderGraph {
        &mut self.graph
    }

    /// Get ref to [`RenderGraph`]
    pub fn graph(&mut self) -> &RenderGraph {
        &self.graph
    }

    /// Draw a frame or skip a frame if some resources need to be create
    /// # Panic!
    ///
    /// If the GPU has stopped responding or an unexpected error occurred
    pub fn draw_frame(&mut self) -> VulkanResult<()> {
        profile_scope!("WorldRenderer::draw_frame");
        GlobalProfiler::lock().new_frame();

        match self.graph.execute(&mut self.ctx, &mut self.resources) {
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
                return Err(err);
            },
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

        self.resources.destroy(device);
        self.camera.destroy(device);
        self.graph.destroy(device);
        self.bindless.destroy(device);

        // Safety: All low-level vulkan resources are destroyed before that
        unsafe { ManuallyDrop::drop(&mut self.ctx) };
    }
}
