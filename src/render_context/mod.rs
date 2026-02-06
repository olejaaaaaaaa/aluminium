use ash::vk;

mod window_manager;
pub use window_manager::WindowManager;

mod graphics_device;
pub use graphics_device::GraphicsDevice;

use crate::core::{
    AppBuilder, CommandPoolBuilder, DeviceBuilder, FrameBufferBuilder, FrameSync, ImageBuilder,
    ImageViewBuilder, InstanceBuilder, PhysicalDevice, QueuePool, RenderPassBuilder,
    SurfaceBuilder, SwapchainBuilder, VulkanResult,
};

/// Render Context provides initialized low-level Vulkan objects ready to use.
pub struct RenderContext {
    /// Window and Swapchain management
    pub(crate) window: WindowManager,
    /// Main Vulkan objects
    pub(crate) device: GraphicsDevice,
}

pub struct RenderContextConfig {
    frame_count: Option<u32>,
    phys_dev_type_priority: Vec<vk::PhysicalDeviceType>,
    surface_format_priority: Vec<vk::Format>,
    depth_format_priority: Vec<vk::Format>,
    present_mode_priority: Vec<vk::PresentModeKHR>,
    max_api_version: u32,
    min_api_version: u32,
}

impl Default for RenderContextConfig {
    fn default() -> Self {
        RenderContextConfig {
            frame_count: None,
            phys_dev_type_priority: vec![
                vk::PhysicalDeviceType::DISCRETE_GPU,
                vk::PhysicalDeviceType::INTEGRATED_GPU,
            ],
            surface_format_priority: vec![vk::Format::R8G8B8A8_SRGB, vk::Format::B8G8R8A8_SRGB],
            depth_format_priority: vec![vk::Format::D32_SFLOAT],
            present_mode_priority: vec![vk::PresentModeKHR::MAILBOX, vk::PresentModeKHR::FIFO],
            max_api_version: vk::API_VERSION_1_0,
            min_api_version: vk::API_VERSION_1_0,
        }
    }
}

impl RenderContext {
    pub fn resize(&mut self, width: u32, height: u32) -> VulkanResult<()> {
        self.window.resize(&self.device, width, height)
    }

    pub fn with_config(
        window: &winit::window::Window,
        config: RenderContextConfig,
    ) -> VulkanResult<RenderContext> {
        let app = AppBuilder::default()
            .with_min_api_version(config.min_api_version)
            .with_max_api_version(config.max_api_version)
            .build()?;

        let instance = InstanceBuilder::default(&app).build()?;
        let surface = SurfaceBuilder::new(&app, &instance, window).build()?;

        let phys_dev = unsafe { instance.raw.enumerate_physical_devices().unwrap() };
        let phys_dev = phys_dev[0];
        let phys_prop = unsafe { instance.raw.get_physical_device_properties(phys_dev) };

        let phys_dev = PhysicalDevice {
            raw: phys_dev,
            prop: phys_prop,
        };

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

    /// Create Render Context with default params
    ///
    /// Render Pass with Depth Image
    ///
    /// Chouse integrated GPU
    ///
    /// Only Vulkan 1.0 and the most necessary extensions
    pub fn new(window: &winit::window::Window) -> VulkanResult<Self> {
        let config = RenderContextConfig::default();
        Self::with_config(window, config)
    }

    // Debug
    fn execute_commands(&self, callback: impl FnOnce(vk::CommandBuffer)) {
        let device = &self.device;

        let queue = self
            .device
            .queue_pool
            .get_queue(vk::QueueFlags::TRANSFER)
            .unwrap();
        let pool = CommandPoolBuilder::reset(device).build().unwrap();
        let cbuf = pool.create_command_buffers(device, 1).unwrap()[0];

        unsafe {
            self.device
                .begin_command_buffer(
                    cbuf,
                    &vk::CommandBufferBeginInfo::default()
                        .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT),
                )
                .unwrap();
        }

        callback(cbuf);

        unsafe {
            self.device.end_command_buffer(cbuf).unwrap();

            let submit_info =
                vk::SubmitInfo::default().command_buffers(std::slice::from_ref(&cbuf));

            self.device
                .queue_submit(queue, &[submit_info], vk::Fence::null())
                .expect("queue submit failed.");

            self.device.device_wait_idle().unwrap();
        }
    }
}

/// Destroying Vulkan objects in the correct order
impl Drop for RenderContext {
    fn drop(&mut self) {
        unsafe { self.device.device_wait_idle().expect("Error wait idle") };

        self.window.swapchain.destroy();

        self.window.render_pass.destroy(&self.device);
        self.window.depth_view.destroy(&self.device);
        self.window.depth_image.destory(&self.device);

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
