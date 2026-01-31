use ash::vk;

use crate::{DescriptorPool, DescriptorPoolBuilder, Device, GpuBuffer, VulkanResult};

const MAX_DESCRIPTOR_SET: usize = 10000;

/// DescriptorManager for create all descriptor set
pub struct DescriptorManager {
    pool: DescriptorPool,
}

impl DescriptorManager {
    pub fn destroy(&self, device: &Device) {
        unsafe {
            device.destroy_descriptor_pool(self.pool.raw, None);
        }
    }

    pub fn new(device: &Device) -> VulkanResult<Self> {
        let pool = DescriptorPoolBuilder::new(device)
            .pool_sizes(&[
                vk::DescriptorPoolSize::default()
                    .descriptor_count(500)
                    .ty(vk::DescriptorType::SAMPLER),
                vk::DescriptorPoolSize::default()
                    .descriptor_count(5000)
                    .ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER),
                vk::DescriptorPoolSize::default()
                    .descriptor_count(5000)
                    .ty(vk::DescriptorType::UNIFORM_BUFFER),
                vk::DescriptorPoolSize::default()
                    .descriptor_count(500)
                    .ty(vk::DescriptorType::STORAGE_BUFFER),
                vk::DescriptorPoolSize::default()
                    .descriptor_count(500)
                    .ty(vk::DescriptorType::STORAGE_IMAGE),
            ])
            .max_sets(100000)
            .build()?;

        Ok(DescriptorManager { pool })
    }
}

impl DescriptorManager {
    pub fn create_descriptor_set(
        &self,
        device: &Device,
        layouts: &[vk::DescriptorSetLayout],
    ) -> Vec<vk::DescriptorSet> {
        self.pool.create_descriptor_set(device, layouts)
    }
}
