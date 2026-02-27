use std::marker::PhantomData;
use std::ops::DerefMut;
use std::sync::{Arc, LazyLock, RwLock};

use ash::vk;
use bytemuck::{Pod, Zeroable};
use puffin::{profile_scope, GlobalProfiler};
use winit::window::Window;

use super::render_context::RenderContext;
use crate::bindless::Bindless;
use crate::camera::{Camera, CameraData};
use crate::core::{
    AttributeDescriptions, BindingDescriptions, Resolution, SwapchainError, VulkanError,
    VulkanResult,
};
use crate::frame_values::{FrameData, FrameValues};
use crate::render_graph::RenderGraph;
use crate::resource_manager::{
    AssetManager, Material, MaterialHandle, MeshHandle, Renderable, RenderableHandle,
    ResourceManager, Transform, TransformHandle,
};

/// A lightweight, performance-oriented abstraction over the Vulkan API
///
/// Prioritizes performance over safety in certain cases - some operations
/// are marked `unsafe` and require the caller to uphold invariants that
/// cannot be enforced at compile time
///
/// # Asset Creation
///
/// Mesh, material, and transform creation is single-threaded from the caller's
/// perspective. Internally, some Vulkan object creation may be parallelized,
/// but this is an implementation detail and is not exposed through the public API
///
/// # Stability
///
/// This abstraction is currently experimental. The public API may change
/// between versions without prior notice
/// 
/// # Example
///
/// ```rust,no_run
/// let renderer = WorldRenderer::new(&window)?;
/// renderer.draw_frame(|graph| { ... });
/// ```
pub struct WorldRenderer {
    /// Contains resources for rendering and caches them
    resources: Arc<ResourceManager>,
    /// Handles pass scheduling, dependency resolution, and automatic resource barriers
    graph: RenderGraph,
    /// View and projection data submitted to the GPU each frame
    camera: Camera,
    /// Owns the Vulkan instance, device, swapchain, and allocator and etc.
    /// Shared across threads; swapchain recreation requires exclusive access
    ctx: Arc<RenderContext>,
    /// Makes `WorldRenderer` neither `Send` nor `Sync`.
    /// Vulkan objects tied to this renderer must not cross thread boundaries
    _marker: PhantomData<*mut ()>,
}

impl WorldRenderer {
    /// Creates a new [`WorldRenderer`]!
    ///
    /// # Panics!
    ///
    /// If Vulkan is not found on this device or the device does not support
    /// core formats or features.
    pub fn new(window: &Window) -> VulkanResult<WorldRenderer> {
        let ctx = RenderContext::new(window)?;
        let camera = Camera::new(&ctx.device)?;
        let resources = ResourceManager::new(ctx.clone())?;
        let graph = RenderGraph::new(ctx.clone(), resources.clone(), &camera)?;

        Ok(WorldRenderer {
            resources,
            camera,
            graph,
            ctx,
            _marker: PhantomData,
        })
    }

    /// Get a reference to [`AssetManager`]
    pub fn with_assets<R, F: FnOnce(&AssetManager) -> R>(&self, closure: F) -> R {
        closure(&self.resources.assets.read().unwrap())
    }

    /// Get mut reference to [`AssetManager`]
    pub fn with_assets_mut<R, F: FnOnce(&mut AssetManager) -> VulkanResult<R>>(&self, closure: F) -> VulkanResult<R> {
        closure(&mut self.resources.assets.write().unwrap())
    }

    /// Gets a mut reference to the [`Camera`]
    pub fn camera_mut(&mut self) -> &mut Camera {
        &mut self.camera
    }

    /// Gets a reference to the [`Camera`]
    pub fn camera(&self) -> &Camera {
        &self.camera
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
        profile_scope!("WorldRenderer::resize");

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
    pub fn draw_frame<R, F: FnOnce(&mut RenderGraph) -> R>(&mut self, setup: F) -> VulkanResult<R> {
        profile_scope!("WorldRenderer::draw_frame");
        GlobalProfiler::lock().new_frame();

        // Setup graph
        let result = setup(&mut self.graph);

        // Compile Graph
        self.graph.compile()?;

        // Execute Graph
        if let Err(err) = self.graph.execute() {
            if let VulkanError::Swapchain(err) = err {
                match err {
                    SwapchainError::SwapchainOutOfDateKhr | SwapchainError::SwapchainSubOptimal => {
                        let extent = self.ctx.resolution();
                        self.resize(extent.width, extent.height)?;
                    },
                    SwapchainError::SwapchainCreationFailed(err) => {
                        return Err(VulkanError::Swapchain(
                            SwapchainError::SwapchainCreationFailed(err),
                        ));
                    },
                }
            } else {
                log::error!("Error execute frame: {:?}", err);
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

        // Destroy Gpu Buffers
        // Destroy Pipelines
        // Destroy Pipeline Layouts
        // Destroy FrameBuffers
        // self.resources.destroy(device);
        // Destroy Uniform Buffer
        self.camera.destroy(device);
        // Destroy CommandPool
        self.graph.destroy(device);
        // Destroy DescriptorPool
        // self.bindless.destroy(device);
        // Destroy Swapchain
        // Destroy RenderPass
        // Destroy DepthView
        // Destroy DepthImage
        // Destroy Frame Sync objects: (Semaphore, Semaphore, Fence)
        // Destroy FrameBuffers
        // Destroy Swapchain ImageViews
        // Destroy Gpu Allocator
        // Destroy Device
        // Destroy Surface
        // Destroy Option<DebugCallback>
        // Destroy Instance
        // unsafe { ManuallyDrop::drop(&mut self.ctx) };
    }
}
