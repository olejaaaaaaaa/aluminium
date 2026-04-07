use crate::{VulkanResult, core::{DescriptorPool, DescriptorPoolBuilder, DescriptorSetLayoutBuilder, Device}};
use ash::vk;

pub struct DescriptorManager {
    pool: DescriptorPool
}

impl DescriptorManager {
    fn new(device: &Device) -> VulkanResult<Self> {

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

        // vk::DescriptorSetLayoutBinding::default()
        //     .

        let layout = DescriptorSetLayoutBuilder::new(device)
            .bindings(vec![
                
            ])
            .build()?;

        let set = pool.create_descriptor_set(device, &[layout.raw])?;

        Ok(Self {
            pool
        })
    }
}