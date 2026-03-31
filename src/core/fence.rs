use ash::vk;
use tracing::debug;

use super::device::Device;
use super::{VulkanError, VulkanResult};
use crate::core::debug;

pub struct Fence {
    pub raw: vk::Fence,
}

pub struct FenceBuilder<'a> {
    pub device: &'a Device,
    pub flags: vk::FenceCreateFlags,
}

impl<'a> FenceBuilder<'a> {
    pub fn signaled(device: &'a Device) -> Self {
        Self {
            device,
            flags: vk::FenceCreateFlags::SIGNALED,
        }
    }

    pub fn build(self) -> VulkanResult<Fence> {
        let create_info = vk::FenceCreateInfo::default().flags(self.flags);

        let fence = unsafe {
            profiling::scope!("vkCreateFence");
            self.device
                .create_fence(&create_info, None)
                .map_err(VulkanError::Unknown)
        }?;

        debug!(
            handle = ?fence,
            flags = ?self.flags,
            "Fence created"
        );

        Ok(Fence { raw: fence })
    }
}
