use std::sync::Arc;

use ash::vk;
use parking_lot::RwLock;

mod window_manager;
use tracing::warn;
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

    /// Recreate [`WindowManager`]
    pub fn resize(&self, width: u32, height: u32) -> VulkanResult<()> {
        self.window.write().resize(&self.device, width, height)
    }

    /// Create [`RenderContext`]
    pub fn new(window: &winit::window::Window) -> VulkanResult<Arc<Self>> {
        let app = App::new()?;
        let instance = Instance::new(window, &app)?;
        let surface = Surface::new(&app, &instance, window)?;
        let phys_dev = PhysicalDevice::new(&instance)?;
        let device = Device::new(&instance, &phys_dev)?;

        let caps = surface.get_physical_device_surface_capabilities(phys_dev.raw)?;
        let extent = caps.current_extent;

        let formats = surface.get_physical_device_surface_formats(phys_dev.raw)?;

        let format_priority = [vk::Format::R8G8B8A8_SRGB];

        let color_space_priority = [
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

        warn!("Image count: {}:{}", caps.min_image_count, caps.max_image_count);

        let swapchain = SwapchainBuilder::new(&device)
            .min_image_count(caps.max_image_count)
            .surface(&surface)
            .present_mode(vk::PresentModeKHR::FIFO)
            .instance(&instance)
            .color_space(color_space)
            .extent(extent)
            .format(format)
            .build()?;

        let render_pass = RenderPassBuilder::default(&device, vk::Format::R8G8B8A8_SRGB, vk::Format::D32_SFLOAT).build()?;

        let depth_image = ImageBuilder::new(&device)
            .extent(caps.current_extent.into())
            .format(vk::Format::D32_SFLOAT)
            .usage(vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT)
            .build()?;

        let depth_view = ImageViewBuilder::new(&device)
            .format(vk::Format::D32_SFLOAT)
            .image(depth_image.raw)
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::DEPTH,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            })
            .view_type(vk::ImageViewType::TYPE_2D)
            .build()?;

        let mut image_views = vec![];

        for i in swapchain.get_swapchain_images().unwrap() {
            let image_view = ImageViewBuilder::new(&device)
                .format(vk::Format::R8G8B8A8_SRGB)
                .image(i)
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                })
                .view_type(vk::ImageViewType::TYPE_2D)
                .build()?;
            image_views.push(image_view);
        }

        let mut frame_buffers = vec![];

        for i in &image_views {
            let frame_buffer = FrameBufferBuilder::new(&device)
                .render_pass(render_pass.raw)
                .attachments(&[i.raw, depth_view.raw])
                .extent(caps.current_extent)
                .layers(1)
                .build()?;

            frame_buffers.push(frame_buffer);
        }

        let pool = QueuePool::new(&device.raw, &phys_dev.raw, &surface, &device.queue_family_props);
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
            let device = &mut self.device;
            let window = &mut self.window.write();

            device
                .device_wait_idle()
                .expect("Failed to wait for device idle during RenderContext drop!");

            for i in window.image_views.drain(..) {
                i.destroy(device);
            }

            window.depth_view.destroy(device);
            window.depth_image.destroy(device);
            window.render_pass.destroy(device);

            for i in window.frame_buffers.drain(..) {
                i.destroy(device);
            }

            for i in window.frame_sync.drain(..) {
                i.destroy(device);
            }

            window.swapchain.destroy();
            window.surface.destroy();
            device.logical_device.destroy();
            device.instance.destroy();
        }
    }
}
