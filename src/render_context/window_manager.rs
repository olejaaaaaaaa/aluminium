use ash::vk;
use log::info;
use puffin::profile_scope;

use super::GraphicsDevice;
use crate::core::{
    FrameBuffer, FrameBufferBuilder, FrameSync, Image, ImageBuilder, ImageView, ImageViewBuilder,
    RenderPass, Surface, Swapchain, SwapchainBuilder, VulkanResult,
};

/// Manages window-related Vulkan resources (swapchain, framebuffers, etc.)
/// Handles window resizing and frame synchronization
pub struct WindowManager {
    /// Current window/swapchain resolution
    pub(crate) resolution: vk::Extent2D,
    /// Synchronization objects for each frame in flight
    pub(crate) frame_sync: Vec<FrameSync>,
    /// Framebuffers (one per swapchain image)
    pub(crate) frame_buffers: Vec<FrameBuffer>,
    /// Image views for swapchain images
    pub(crate) image_views: Vec<ImageView>,
    /// Index of current frame
    pub(crate) current_frame: usize,
    /// Depth buffer image
    pub(crate) depth_image: Image,
    /// View into the depth buffer
    pub(crate) depth_view: ImageView,
    /// Render pass defining attachment formats and operations
    pub(crate) render_pass: RenderPass,
    /// Window surface for presentation
    pub(crate) surface: Surface,
    /// Swapchain containing presentable images
    pub(crate) swapchain: Swapchain,
}

impl WindowManager {
    /// Recreate swapchain, image views, depth image, framebuffers for new
    /// window size
    pub fn resize(&mut self, device: &GraphicsDevice, width: u32, height: u32) -> VulkanResult<()> {
        profile_scope!("WindowManager::resize");

        info!("New size: {:?}", (width, height));

        unsafe { device.device_wait_idle().expect("Error wait idle") };

        let caps = self
            .surface
            .get_physical_device_surface_capabilities(*device.phys_dev);

        let swapchain = SwapchainBuilder::default(&device.instance, &device.device, &self.surface)
            .old_swapchain(self.swapchain.raw)
            .extent(caps.current_extent)
            .format(vk::Format::R8G8B8A8_SRGB)
            .build()?;

        let depth_image =
            ImageBuilder::depth(device, vk::Format::D32_SFLOAT, caps.current_extent).build()?;

        let depth_view =
            ImageViewBuilder::depth(device, vk::Format::D32_SFLOAT, depth_image.raw).build()?;

        let mut image_views = vec![];

        for i in swapchain.get_swapchain_images()? {
            let image_view =
                ImageViewBuilder::new_2d(device, vk::Format::R8G8B8A8_SRGB, i).build()?;

            image_views.push(image_view);
        }

        let mut frame_buffers = vec![];

        for i in &image_views {
            let frame_buffer = FrameBufferBuilder::new(device, self.render_pass.raw)
                .add_attachment(i.raw)
                .add_attachment(depth_view.raw)
                .extent(caps.current_extent)
                .layers(1)
                .build()?;

            frame_buffers.push(frame_buffer);
        }

        self.depth_view.destroy(device);
        self.depth_view = depth_view;

        self.depth_image.destroy(device);
        self.depth_image = depth_image;

        for i in &self.image_views {
            i.destroy(device);
        }

        self.image_views = image_views;

        for i in &self.frame_buffers {
            i.destroy(device);
        }

        self.frame_buffers = frame_buffers;

        self.swapchain.destroy();
        self.swapchain = swapchain;

        self.resolution = caps.current_extent;

        Ok(())
    }
}
