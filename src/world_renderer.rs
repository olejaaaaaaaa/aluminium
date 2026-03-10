use std::io::Read;
use std::marker::PhantomData;
use std::ops::DerefMut;
use std::sync::{Arc, LazyLock, RwLock};

use ash::vk;
use bytemuck::{Pod, Zeroable};
use log::warn;
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
    AssetManager, Create, Get, GetMut, Handle, Material, MaterialHandle, MeshHandle, Renderable, RenderableHandle, Resources, Transform, TransformHandle
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
/// but this is an implementation detail and is not exposed through the public
/// API
///
/// # Stability
///
/// This abstraction is currently experimental. The public API may change
/// between versions without prior notice
///
/// # Example
///
/// ```ignore
/// let renderer = WorldRenderer::new(&window)?;
/// renderer.draw_frame(|graph| { ... });
/// ```
pub struct WorldRenderer {
    /// Contains resources for rendering and caches them
    resources: Arc<Resources>,
    /// Handles pass scheduling, dependency resolution, and automatic resource
    /// barriers
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
        let resources = Resources::new(ctx.clone())?;
        let graph = RenderGraph::new(ctx.clone(), resources.clone(), &camera)?;

        Ok(WorldRenderer {
            resources,
            camera,
            graph,
            ctx,
            _marker: PhantomData,
        })
    }

    /// Create a new resource of type `T` with the given description and return a handle to it
    pub fn create<T: Create>(&mut self, desc: T::Desc) -> VulkanResult<Handle<T>> {
        T::create(&self.ctx, &self.resources, desc)
    }

    /// Get a mutable reference to a resource of type `T` with the given handle, or panic if it doesn't exist
    pub fn get_mut<T: GetMut>(&mut self, handle: Handle<T>) -> &mut T {
        self.try_get_mut(handle).unwrap()
    }

    /// Try to get a mutable reference to a resource of type `T` with the given handle
    pub fn try_get_mut<T: GetMut>(&mut self, handle: Handle<T>) -> Option<&mut T> {
        T::try_get_mut(&self.resources, handle)
    }

    /// Get a reference to a resource of type `T` with the given handle, or panic if it doesn't exist
    pub fn get<T: Get>(&mut self, handle: Handle<T>) -> &T {
        self.try_get(handle).unwrap()
    }

    /// Try to get a reference to a resource of type `T` with the given handle
    pub fn try_get<T: Get>(&mut self, handle: Handle<T>) -> Option<&T> {
        T::try_get(&self.resources, handle)
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
                    SwapchainError::SwapchainOutOfDateKhr => {
                        let extent = self.ctx.window.read().unwrap().resolution;
                        self.resize(extent.width, extent.height)?;
                    },
                    SwapchainError::SwapchainCreationFailed(err) => {
                        return Err(VulkanError::Swapchain(
                            SwapchainError::SwapchainCreationFailed(err),
                        ));
                    },
                    _ => {
                        warn!("Swapchain error during frame execution: {:?}", err);
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
        // Destroy CommandPool
        // Destroy Gpu Buffers
        // Destroy Pipelines
        // Destroy Pipeline Layouts
        // Destroy FrameBuffers
        // Destroy Uniform Buffer
        // Destroy DescriptorPool
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
    }
}
