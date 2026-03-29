use ash::vk;

use super::{Device, VulkanError, VulkanResult};

pub struct Semaphore {
    pub raw: vk::Semaphore,
}

pub struct SemaphoreBuilder<'a> {
    device: &'a Device,
}

impl<'a> SemaphoreBuilder<'a> {
    pub fn new(device: &'a Device) -> Self {
        Self { device }
    }

    pub fn build(self) -> VulkanResult<Semaphore> {
        let create_info = vk::SemaphoreCreateInfo::default();

        let semaphore = unsafe {
            profiling::scope!("vkCreateSemaphore");
            self.device
                .create_semaphore(&create_info, None)
                .map_err(VulkanError::Unknown)
        }?;

        Ok(Semaphore { raw: semaphore })
    }
}
