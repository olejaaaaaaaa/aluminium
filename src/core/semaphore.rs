use ash::vk;

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

        let sem = unsafe {
            profiling::scope!("vkCreateSemaphore");
            self.device
                .create_semaphore(&self.create_info, None)
                .map_err(VulkanError::Unknown)
        }?;

        Ok(Semaphore { raw: sem })
    }
}
