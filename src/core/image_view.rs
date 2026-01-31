use ash::vk;

use super::device::Device;
use super::{VulkanError, VulkanResult};

pub struct ImageView {
    pub(crate) raw: vk::ImageView,
}

impl ImageView {
    pub fn destroy(&self, device: &Device) {
        unsafe { device.destroy_image_view(self.raw, None) };
    }
}

pub struct ImageViewBuilder<'a> {
    device: &'a Device,
    create_info: vk::ImageViewCreateInfo<'static>,
}

impl<'a> ImageViewBuilder<'a> {
    pub fn format(mut self, format: vk::Format) -> Self {
        self.create_info = self.create_info.format(format);
        self
    }

    pub fn image(mut self, image: vk::Image) -> Self {
        self.create_info = self.create_info.image(image);
        self
    }

    pub fn new_2d(device: &'a Device, format: vk::Format, image: vk::Image) -> Self {
        Self {
            device,
            create_info: vk::ImageViewCreateInfo::default()
                .components(vk::ComponentMapping::default())
                .format(format)
                .image(image)
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                })
                .view_type(vk::ImageViewType::TYPE_2D),
        }
    }

    pub fn depth(device: &'a Device, format: vk::Format, image: vk::Image) -> Self {
        Self {
            device,
            create_info: vk::ImageViewCreateInfo::default()
                .components(vk::ComponentMapping::default())
                .image(image)
                .format(format)
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::DEPTH,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                })
                .view_type(vk::ImageViewType::TYPE_2D),
        }
    }

    pub fn build(self) -> VulkanResult<ImageView> {
        puffin::profile_scope!("ImageView");

        let image_view = unsafe {
            self.device
                .create_image_view(&self.create_info, None)
                .map_err(|e| VulkanError::Unknown(e))?
        };

        Ok(ImageView { raw: image_view })
    }
}
