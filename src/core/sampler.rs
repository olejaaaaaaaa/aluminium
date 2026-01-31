use ash::vk;
use puffin::profile_scope;

use super::{Device, VulkanError, VulkanResult};

pub struct Sampler {
    pub raw: vk::Sampler,
}

impl Sampler {
    pub fn destroy(&self, device: &Device) {
        unsafe { device.destroy_sampler(self.raw, None) };
    }
}

pub struct SamplerBuilder<'a> {
    sampler_info: vk::SamplerCreateInfo<'static>,
    device: &'a Device,
}

impl<'a> SamplerBuilder<'a> {
    pub fn default(device: &'a Device) -> Self {
        Self {
            sampler_info: vk::SamplerCreateInfo::default(),
            device,
        }
    }

    pub fn build(self) -> VulkanResult<Sampler> {
        profile_scope!("Sampler");

        let sampler = unsafe {
            self.device
                .create_sampler(&self.sampler_info, None)
                .map_err(|e| VulkanError::Unknown(e))?
        };

        Ok(Sampler { raw: sampler })
    }
}
