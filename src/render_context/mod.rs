use ash::vk;

mod window_manager;
pub use window_manager::WindowManager;

mod graphics_device;
pub use graphics_device::GraphicsDevice;

use crate::core::{
    App, DeviceBuilder, FrameBufferBuilder, FrameSync, ImageBuilder, ImageViewBuilder,
    Instance, PhysicalDevice, QueuePool, RenderPassBuilder, SurfaceBuilder,
    SwapchainBuilder, VulkanResult,
};

/// Render Context provides initialized low-level Vulkan objects ready to use.
pub struct RenderContext {
    /// Window and Swapchain management
    pub(crate) window: WindowManager,
    /// Main Vulkan objects
    pub(crate) device: GraphicsDevice,
}

impl RenderContext {
    /// Recreate
    /// - `Swapchain`
    /// - `FrameBuffers`
    /// - `ImageViews`
    pub fn resize(&mut self, width: u32, height: u32) -> VulkanResult<()> {
        self.window.resize(&self.device, width, height)
    }

    /// Create Render Context
    pub fn new(window: &winit::window::Window) -> VulkanResult<RenderContext> {

        let app = App::new()?;
        let instance = Instance::new(window, &app)?;
        let surface = SurfaceBuilder::new(&app, &instance, window).build()?;
        let phys_dev = PhysicalDevice::new(&instance)?;
        let device = DeviceBuilder::default(&instance, phys_dev).build()?;
        let caps = surface.get_physical_device_surface_capabilities(*device.phys_dev);
        
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

        Ok(Self {
            window: WindowManager {
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
            },
            device: GraphicsDevice {
                app,
                instance,
                device,
                queue_pool: pool,
            },
        })
    }
}

/// Destroying Vulkan objects in the correct order
impl Drop for RenderContext {
    fn drop(&mut self) {
        unsafe { self.device.device_wait_idle().expect("Error wait idle") };

        self.window.swapchain.destroy();

        self.window.render_pass.destroy(&self.device);
        self.window.depth_view.destroy(&self.device);
        self.window.depth_image.destroy(&self.device);

        for i in &self.window.frame_sync {
            i.destroy(&self.device);
        }

        for i in &self.window.frame_buffers {
            i.destroy(&self.device);
        }

        for i in &self.window.image_views {
            i.destroy(&self.device);
        }

        self.device.device.destroy();
        self.window.surface.destroy();
        self.device.instance.destroy();
    }
}
