use ash::vk;
use puffin::profile_scope;

use super::{Device, VulkanError, VulkanResult};

pub struct Sampler {
    #[allow(dead_code)]
    pub raw: vk::Sampler,
}

impl Sampler {
    #[allow(dead_code)]
    pub fn destroy(&self, device: &Device) {
        unsafe { device.destroy_sampler(self.raw, None) };
    }
}

#[allow(dead_code)]
pub struct SamplerBuilder<'a> {
    sampler_info: vk::SamplerCreateInfo<'static>,
    device: &'a Device,
}

impl<'a> SamplerBuilder<'a> {
    #[allow(dead_code)]
    pub fn default(device: &'a Device) -> Self {
        Self {
            sampler_info: vk::SamplerCreateInfo::default(),
            device,
        }
    }

    #[allow(dead_code)]
    pub fn build(self) -> VulkanResult<Sampler> {
        profile_scope!("Sampler");

        let sampler = unsafe {
            self.device
                .create_sampler(&self.sampler_info, None)
                .map_err(VulkanError::Unknown)?
        };

        Ok(Sampler { raw: sampler })
    }
}
