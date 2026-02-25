use std::sync::{Arc, RwLock};

use ash::vk;

mod features;
pub use features::*;

mod window_manager;
pub use window_manager::WindowManager;

mod graphics_device;
pub use graphics_device::GraphicsDevice;

use crate::core::{
    App, Device, Extension, FrameBufferBuilder, FrameSync, ImageBuilder, ImageViewBuilder,
    Instance, PhysicalDevice, PhysicalFeature, QueuePool, RenderPassBuilder, SurfaceBuilder,
    SwapchainBuilder, VulkanResult,
};

/// Render Context provides initialized low-level Vulkan objects ready to use.
pub struct RenderContext {
    /// Window and Swapchain management
    pub(crate) window: RwLock<WindowManager>,
    /// Main Vulkan objects
    pub(crate) device: GraphicsDevice,
}

impl RenderContext {
    /// Recreate
    /// - `Swapchain`
    /// - `FrameBuffers`
    /// - `ImageViews`
    pub fn resize(&self, width: u32, height: u32) -> VulkanResult<()> {
        self.window.write().expect("Error lock write").resize(&self.device, width, height)
    }

    pub fn resolution(&self) -> vk::Extent2D {
        self.window.read().expect("Error lock Window Manager").resolution
    }

    pub fn framebuffer_count(&self) -> usize {
        self.window.read().expect("Error read lock Window Manager").frame_buffers.len()
    }

    pub fn check_features<T>(&self, features: &[T]) -> bool
    where
        for<'a> &'a T: Into<Feature>,
    {
        for i in features {
            let feature: Feature = i.into();
            match feature {
                Feature::Extension(ext) => if let Extension::Bindless = ext {
                    let extensions = &self.device.logical_device.extensions;
                    if extensions.contains(&c"VK_KHR_push_descriptor")
                        && extensions.contains(&c"VK_KHR_push_descriptor")
                    {
                        return true;
                    }
                },
                Feature::Physical(phys) => match phys {
                    PhysicalFeature::PushConstant256 => {
                        let limits = self.device.phys_dev.prop.limits;
                        if limits.max_push_constants_size == 256 {
                            return true;
                        }
                    },
                },
                Feature::Vendor(v) => {
                    let vendor = self.device.phys_dev.vendor;
                    if vendor == v {
                        return true;
                    }
                },
            }
        }
        false
    }

    /// Create Render Context
    pub fn new(window: &winit::window::Window) -> VulkanResult<Arc<Self>> {
        let app = App::new()?;
        let instance = Instance::new(window, &app)?;
        let surface = SurfaceBuilder::new(&app, &instance, window).build()?;
        let phys_dev = PhysicalDevice::new(&instance)?;
        let device = Device::new(&instance, &phys_dev)?;
        let caps = surface.get_physical_device_surface_capabilities(phys_dev.raw);

        let swapchain = SwapchainBuilder::default(&instance, &device, &surface)
            .extent(caps.current_extent)
            .format(vk::Format::R8G8B8A8_SRGB)
            .build()?;

        let render_pass =
            RenderPassBuilder::default(&device, vk::Format::R8G8B8A8_SRGB, vk::Format::D32_SFLOAT)
                .build()?;

        let depth_image =
            ImageBuilder::depth(&device, vk::Format::D32_SFLOAT, caps.current_extent).build()?;

        let depth_view =
            ImageViewBuilder::depth(&device, vk::Format::D32_SFLOAT, depth_image.raw).build()?;

        let mut image_views = vec![];

        for i in swapchain.get_swapchain_images().unwrap() {
            let image_view =
                ImageViewBuilder::new_2d(&device, vk::Format::R8G8B8A8_SRGB, i).build()?;
            image_views.push(image_view);
        }

        let mut frame_buffers = vec![];

        for i in &image_views {
            let frame_buffer = FrameBufferBuilder::new(&device, render_pass.raw)
                .add_attachment(i.raw)
                .add_attachment(depth_view.raw)
                .extent(caps.current_extent)
                .layers(1)
                .build()?;

            frame_buffers.push(frame_buffer);
        }

        let pool = QueuePool::new(&device.raw, &device.queue_family_props);
        let mut frame_sync = vec![];

        for _ in 0..frame_buffers.len() {
            frame_sync.push(FrameSync::new(&device)?);
        }

        Ok(Arc::new(Self {
            window: RwLock::new(WindowManager {
                resolution: caps.current_extent,
                frame_sync,
                image_views,
                depth_image,
                frame_buffers,
                depth_view,
                current_frame: 0,
                surface,
                swapchain,
                render_pass,
            }),
            device: GraphicsDevice {
                app,
                phys_dev,
                instance,
                logical_device: device,
                queue_pool: pool,
            },
        }))
    }
}

/// Destroying Vulkan objects in the correct order
impl Drop for RenderContext {
    fn drop(&mut self) {
        unsafe { self.device.device_wait_idle().expect("Error wait idle") };

        // self.window.swapchain.destroy();

        // self.window.render_pass.destroy(&self.device);
        // self.window.depth_view.destroy(&self.device);
        // self.window.depth_image.destroy(&self.device);

        // for i in &self.window.frame_sync {
        //     i.destroy(&self.device);
        // }

        // for i in &self.window.frame_buffers {
        //     i.destroy(&self.device);
        // }

        // for i in &self.window.image_views {
        //     i.destroy(&self.device);
        // }

        // self.device.logical_device.destroy();
        // self.window.surface.destroy();
        // self.device.instance.destroy();
    }
}
