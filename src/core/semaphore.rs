use ash::vk;
use puffin::profile_scope;

use super::{Device, VulkanError, VulkanResult};

pub struct Semaphore {
    pub(crate) raw: vk::Semaphore,
}

pub struct SemaphoreBuilder<'a> {
    device: &'a Device,
    create_info: vk::SemaphoreCreateInfo<'static>,
}

impl<'a> SemaphoreBuilder<'a> {
    pub fn new(device: &'a Device) -> Self {
        Self {
            device,
            create_info: vk::SemaphoreCreateInfo::default(),
        }
    }

    pub fn build(self) -> VulkanResult<Semaphore> {
        profile_scope!("Semaphore");

        let sem = unsafe {
            self.device
                .create_semaphore(&self.create_info, None)
                .map_err(|e| VulkanError::Unknown(e))
        }?;

        Ok(Semaphore { raw: sem })
    }
}
