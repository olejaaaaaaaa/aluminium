use std::io::Read;
use std::marker::PhantomData;
use std::ops::DerefMut;
use std::sync::{Arc, LazyLock, RwLock};

use ash::vk;
use bytemuck::{Pod, Zeroable};
use log::warn;
use winit::window::Window;

use super::render_context::RenderContext;
use crate::bindless::{self, Bindless};
use crate::camera::{Camera, CameraData};
use crate::core::{AttributeDescriptions, BindingDescriptions, Resolution, SwapchainError, VulkanError, VulkanResult};
use crate::frame_graph::FrameGraph;
use crate::frame_values::FrameValues;
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
    /// Main Context for creation and some resources
    ctx: Arc<RenderContext>,
    /// Contains resources for rendering and caches them creation
    resources: Arc<Resources>,
    /// Handles pass scheduling, dependency resolution, and automatic
    /// resource barriers
    graph: FrameGraph,
    /// No Send and Sync for this abstraction
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

    /// Create a new resource of type `T` with the given description
    /// and return a handle to it
    pub fn create<T: Create>(&mut self, desc: T::Desc<'_>) -> VulkanResult<Res<T>> {
        T::create(&self.resources, desc)
    }

    /// Get ref
    pub fn get<T: Get>(&self, res: &Res<T>) -> &T {
        T::get(&self.resources, res)
    }

    /// Get mut ref
    pub fn get_mut<T: GetMut>(&mut self, res: &Res<T>) -> &mut T {
        T::get_mut(&self.resources, res)
    }

    /// Gets a mut reference to the [`Camera`]
    pub fn camera_mut(&mut self) -> RefMut<'_, Camera> {
        RefMut(self.resources.camera.write())
    }

    /// Gets a reference to the [`Camera`]
    pub fn camera(&self) -> Ref<'_, Camera> {
        Ref(self.resources.camera.read())
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

    /// Draw a frame or skip a frame if some resources need to be
    /// create
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
                    _ => {
                        log::error!("Swapchain error during frame execution: {:?}", err);
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
    }
}
