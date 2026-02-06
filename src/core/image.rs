use ash::vk;
use puffin::profile_scope;
#[cfg(all(feature = "vma", not(feature = "gpu-allocator")))]
use vk_mem::Alloc;

use super::device::Device;
use super::{VulkanError, VulkanResult};

pub struct Image {
    pub(crate) raw: vk::Image,
    #[cfg(all(feature = "vma", not(feature = "gpu-allocator")))]
    allocation: vk_mem::Allocation,
}

impl Image {
    pub fn destory(&mut self, device: &Device) {
        unsafe {
            device
                .allocator
                .destroy_image(self.raw, &mut self.allocation);
        }
    }
}

pub struct ImageBuilder<'a> {
    pub device: &'a Device,
    pub create_info: vk::ImageCreateInfo<'static>,
    #[cfg(all(feature = "vma", not(feature = "gpu-allocator")))]
    pub alloc_info: vk_mem::AllocationCreateInfo,
}

impl<'a> ImageBuilder<'a> {
    pub fn depth(device: &'a Device, format: vk::Format, extent: vk::Extent2D) -> Self {
        Self {
            device,
            create_info: vk::ImageCreateInfo::default()
                .mip_levels(1)
                .initial_layout(vk::ImageLayout::UNDEFINED)
                .extent(extent.into())
                .format(format)
                .tiling(vk::ImageTiling::OPTIMAL)
                .samples(vk::SampleCountFlags::TYPE_1)
                .array_layers(1)
                .sharing_mode(vk::SharingMode::EXCLUSIVE)
                .usage(vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT)
                .image_type(vk::ImageType::TYPE_2D),
            alloc_info: vk_mem::AllocationCreateInfo {
                usage: vk_mem::MemoryUsage::AutoPreferDevice,
                ..Default::default()
            },
        }
    }

    #[allow(dead_code)]
    pub fn cube(device: &'a Device, format: vk::Format, extent: vk::Extent2D) -> Self {
        Self {
            device,
            create_info: vk::ImageCreateInfo::default()
                .mip_levels(1)
                .initial_layout(vk::ImageLayout::UNDEFINED)
                .extent(extent.into())
                .format(format)
                .tiling(vk::ImageTiling::OPTIMAL)
                .samples(vk::SampleCountFlags::TYPE_1)
                .array_layers(6)
                .sharing_mode(vk::SharingMode::EXCLUSIVE)
                .usage(vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT)
                .image_type(vk::ImageType::TYPE_2D)
                .flags(vk::ImageCreateFlags::CUBE_COMPATIBLE),
            alloc_info: vk_mem::AllocationCreateInfo {
                usage: vk_mem::MemoryUsage::AutoPreferDevice,
                ..Default::default()
            },
        }
    }

    #[allow(dead_code)]
    pub fn new_2d(device: &'a Device, format: vk::Format, extent: vk::Extent2D) -> Self {
        Self {
            device,
            create_info: vk::ImageCreateInfo::default()
                .mip_levels(1)
                .initial_layout(vk::ImageLayout::UNDEFINED)
                .extent(extent.into())
                .format(format)
                .tiling(vk::ImageTiling::OPTIMAL)
                .samples(vk::SampleCountFlags::TYPE_1)
                .array_layers(1)
                .sharing_mode(vk::SharingMode::EXCLUSIVE)
                .usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
                .image_type(vk::ImageType::TYPE_2D),
            alloc_info: vk_mem::AllocationCreateInfo {
                usage: vk_mem::MemoryUsage::AutoPreferDevice,
                ..Default::default()
            },
        }
    }

    #[allow(dead_code)]
    pub fn usage(mut self, usage: vk::ImageUsageFlags) -> Self {
        self.create_info = self.create_info.usage(usage);
        self
    }

    pub fn build(self) -> VulkanResult<Image> {
        profile_scope!("Image");

        let (image, allocation) = unsafe {
            self.device
                .allocator
                .create_image(&self.create_info, &self.alloc_info)
                .map_err(|e| VulkanError::Unknown(e))?
        };

        Ok(Image {
            raw: image,
            allocation,
        })
    }
}
