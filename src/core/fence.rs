use ash::vk;
use puffin::profile_scope;

use super::device::Device;
use super::{VulkanError, VulkanResult};

pub struct Fence {
    pub raw: vk::Fence,
}

pub struct FenceBuilder<'a> {
    pub device: &'a Device,
    pub create_info: vk::FenceCreateInfo<'static>,
}

impl<'a> FenceBuilder<'a> {
    pub fn signaled(device: &'a Device) -> Self {
        Self {
            device,
            create_info: vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED),
        }
    }

    pub fn build(self) -> VulkanResult<Fence> {
        profile_scope!("Fence");

        let fence = unsafe {
            self.device
                .create_fence(&self.create_info, None)
                .map_err(|e| VulkanError::Unknown(e))
        }?;

        Ok(Fence { raw: fence })
    }
}
