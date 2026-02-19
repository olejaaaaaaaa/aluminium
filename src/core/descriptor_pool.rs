use ash::vk;
use puffin::profile_scope;

use super::device::Device;
use super::{VulkanError, VulkanResult};

pub struct DescriptorPool {
    pub raw: vk::DescriptorPool,
}

impl DescriptorPool {
    pub fn create_descriptor_set(
        &self,
        device: &Device,
        layouts: &[vk::DescriptorSetLayout],
    ) -> Vec<vk::DescriptorSet> {
        profile_scope!("Descriptor Set");

        let desc = vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(self.raw)
            .set_layouts(layouts);

        unsafe { device.allocate_descriptor_sets(&desc).unwrap() }
    }
}

pub struct DescriptorPoolBuilder<'a> {
    device: &'a Device,
    create_info: vk::DescriptorPoolCreateInfo<'a>,
}

impl<'a> DescriptorPoolBuilder<'a> {
    pub fn new(device: &'a Device) -> Self {
        Self {
            device,
            create_info: vk::DescriptorPoolCreateInfo::default(),
        }
    }

    pub fn pool_sizes(mut self, sizes: &'a [vk::DescriptorPoolSize]) -> Self {
        self.create_info = self.create_info.pool_sizes(sizes);
        self
    }

    pub fn max_sets(mut self, sets: u32) -> Self {
        self.create_info = self.create_info.max_sets(sets);
        self
    }

    pub fn build(self) -> VulkanResult<DescriptorPool> {
        profile_scope!("DescriptorPool");

        let pool = unsafe {
            self.device
                .create_descriptor_pool(&self.create_info, None)
                .map_err(VulkanError::Unknown)
        }?;

        Ok(DescriptorPool { raw: pool })
    }
}
