use std::sync::Arc;

use ash::vk;
use parking_lot::RwLock;

mod window_manager;
pub use window_manager::WindowManager;

mod graphics_device;
pub use graphics_device::GraphicsDevice;

use crate::core::{
    App, Device, FrameBufferBuilder, FrameSync, ImageBuilder, ImageViewBuilder, Instance, PhysicalDevice, QueuePool, RenderPassBuilder, Surface,
    SwapchainBuilder, VulkanResult,
};

/// Render Context provides initialized low-level Vulkan objects ready
/// to use
pub struct RenderContext {
    /// Window and Swapchain management
    pub(crate) window: RwLock<WindowManager>,
    /// Main Vulkan objects
    pub(crate) device: GraphicsDevice,
}

impl RenderContext {
    pub fn frame_count(&self) -> usize {
        self.window.read().frame_buffers.len()
    }

    pub fn resolution(&self) -> vk::Extent2D {
        self.window.read().resolution
    }

    /// Recreate
    /// - `Swapchain`
    /// - `FrameBuffers`
    /// - `ImageViews`
    pub fn resize(&self, width: u32, height: u32) -> VulkanResult<()> {
        self.window.write().resize(&self.device, width, height)
    }

    /// Create Render Context
    pub fn new(window: &winit::window::Window) -> VulkanResult<Arc<Self>> {
        let app = App::new()?;
        let instance = Instance::new(window, &app)?;
        let surface = Surface::new(&app, &instance, window)?;
        let phys_dev = PhysicalDevice::new(&instance)?;
        let device = Device::new(&instance, &phys_dev)?;

        let caps = surface.get_physical_device_surface_capabilities(phys_dev.raw)?;
        let extent = caps.current_extent;
        let transforms = caps.current_transform;

        let formats = surface.get_physical_device_surface_formats(phys_dev.raw)?;

        let format_priority = [vk::Format::R8G8B8A8_SRGB];

        let color_space_priority = [
            #[cfg(target_os = "android")]
            vk::ColorSpaceKHR::EXTENDED_SRGB_LINEAR_EXT,
            vk::ColorSpaceKHR::SRGB_NONLINEAR,
        ];

        let mut format = vk::Format::R8G8B8A8_SRGB;
        let mut color_space = vk::ColorSpaceKHR::SRGB_NONLINEAR;

        for (f, c) in format_priority.iter().zip(color_space_priority.iter()) {
            for j in &formats {
                if *f == j.format && *c == j.color_space {
                    format = j.format;
                    color_space = j.color_space;
                }
            }
        }

        let swapchain = SwapchainBuilder::new()
            .min_image_count(caps.min_image_count)
            .surface(&surface)
            .present_mode(vk::PresentModeKHR::FIFO)
            .instance(&instance)
            .device(&device)
            .color_space(color_space)
            .extent(extent)
            .format(format)
            .build()?;

        let render_pass = RenderPassBuilder::default(&device, vk::Format::R8G8B8A8_SRGB, vk::Format::D32_SFLOAT).build()?;
        let depth_image = ImageBuilder::depth(&device, vk::Format::D32_SFLOAT, caps.current_extent).build()?;
        let depth_view = ImageViewBuilder::depth(&device, vk::Format::D32_SFLOAT, depth_image.raw).build()?;

        let mut image_views = vec![];

        for i in swapchain.get_swapchain_images().unwrap() {
            let image_view = ImageViewBuilder::new_2d(&device, vk::Format::R8G8B8A8_SRGB, i).build()?;
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

impl Drop for RenderContext {
    fn drop(&mut self) {
        unsafe {
            let device = &self.device.logical_device;
            let window = &mut self.window.write();
            unsafe {
                device
                    .device_wait_idle()
                    .expect("Failed to wait for device idle during RenderContext drop!");
            }
        }
    }
}
