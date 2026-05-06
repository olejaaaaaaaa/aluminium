use ash::vk;

use crate::core::{DescriptorPool, DescriptorPoolBuilder, DescriptorSetLayoutBuilder, Device};
use crate::VulkanResult;

pub struct DescriptorManager {
    pub pool: DescriptorPool,
}

impl DescriptorManager {
    pub fn new(device: &Device) -> VulkanResult<Self> {
        let pool = DescriptorPoolBuilder::new(device)
            .pool_sizes(&[
                vk::DescriptorPoolSize::default()
                    .ty(vk::DescriptorType::UNIFORM_BUFFER)
                    .descriptor_count(10),
                vk::DescriptorPoolSize::default()
                    .ty(vk::DescriptorType::SAMPLER)
                    .descriptor_count(10),
                vk::DescriptorPoolSize::default()
                    .ty(vk::DescriptorType::STORAGE_BUFFER)
                    .descriptor_count(10),
                vk::DescriptorPoolSize::default()
                    .ty(vk::DescriptorType::STORAGE_IMAGE)
                    .descriptor_count(10),
            ])
            .max_sets(10)
            .build()?;

        Ok(Self { pool })
    }
}
