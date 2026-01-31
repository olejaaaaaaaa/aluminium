use ash::vk;
use log::debug;
use puffin::profile_scope;

use super::device::Device;
use super::{Instance, Surface, VulkanError, VulkanResult};

pub struct Swapchain {
    pub(crate) raw: vk::SwapchainKHR,
    pub(crate) loader: ash::khr::swapchain::Device,
}

pub struct SwapchainBuilder<'a> {
    create_info: vk::SwapchainCreateInfoKHR<'static>,
    surface: &'a Surface,
    instance: &'a Instance,
    device: &'a Device,
}

impl<'a> SwapchainBuilder<'a> {
    pub fn default(instance: &'a Instance, device: &'a Device, surface: &'a Surface) -> Self {
        SwapchainBuilder {
            surface,
            instance,
            device,
            create_info: vk::SwapchainCreateInfoKHR::default()
                .clipped(true)
                .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
                .pre_transform(vk::SurfaceTransformFlagsKHR::IDENTITY)
                .present_mode(vk::PresentModeKHR::FIFO)
                .image_array_layers(1)
                .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
                .min_image_count(2),
        }
    }

    pub fn old_swapchain(mut self, swapchian: vk::SwapchainKHR) -> Self {
        self.create_info = self.create_info.old_swapchain(swapchian);
        self
    }

    pub fn format(mut self, format: vk::Format) -> Self {
        self.create_info = self.create_info.image_format(format);
        self
    }

    pub fn extent(mut self, extent: vk::Extent2D) -> Self {
        self.create_info = self.create_info.image_extent(extent);
        self
    }

    pub fn build(self) -> VulkanResult<Swapchain> {
        profile_scope!("Swapchain");

        let create_info = self.create_info.surface(self.surface.raw);

        debug!("Swapchain: {:?}", create_info);

        let swapchain_loader =
            ash::khr::swapchain::Device::new(&self.instance.raw, &self.device.raw);

        let swapchain = unsafe {
            swapchain_loader
                .create_swapchain(&create_info, None)
                .map_err(|e| VulkanError::Unknown(e))
        }?;

        Ok(Swapchain {
            raw: swapchain,
            loader: swapchain_loader,
        })
    }
}

impl Swapchain {
    pub fn get_swapchain_images(&self) -> VulkanResult<Vec<vk::Image>> {
        unsafe {
            self.loader
                .get_swapchain_images(self.raw)
                .map_err(|e| VulkanError::Unknown(e))
        }
    }

    pub fn destroy(&self) {
        unsafe {
            self.loader.destroy_swapchain(self.raw, None);
        }
    }
}
