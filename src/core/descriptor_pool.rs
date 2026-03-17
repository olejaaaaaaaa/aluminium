use ash::vk;
use super::device::Device;
use super::{VulkanError, VulkanResult};

pub struct DescriptorPool {
    pub raw: vk::DescriptorPool,
}

impl DescriptorPool {
    pub fn create_descriptor_set(&self, device: &Device, layouts: &[vk::DescriptorSetLayout]) -> VulkanResult<Vec<vk::DescriptorSet>> {
        
        let desc = vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(self.raw)
            .set_layouts(layouts);

        unsafe { 
            profiling::scope!("vkCreateDescriptorSet");
            device.allocate_descriptor_sets(&desc).map_err(VulkanError::Unknown)
        }
    }
}

pub struct DescriptorPoolBuilder<'a> {
    device: &'a Device,
    flags: vk::DescriptorPoolCreateFlags,
    sizes: Option<&'a [vk::DescriptorPoolSize]>,
    max_sets: Option<u32>,
    create_info: vk::DescriptorPoolCreateInfo<'a>,
}

impl<'a> DescriptorPoolBuilder<'a> {
    pub fn new(device: &'a Device) -> Self {
        Self {
            device,
            flags: vk::DescriptorPoolCreateFlags::empty(),
            sizes: None,
            max_sets: None,
            create_info: vk::DescriptorPoolCreateInfo::default(),
        }
    }

    pub fn flags(mut self, flags: vk::DescriptorPoolCreateFlags) -> Self {
        self.flags = flags;
        self
    }

    pub fn pool_sizes(mut self, sizes: &'a [vk::DescriptorPoolSize]) -> Self {
        self.sizes = Some(sizes);
        self
    }

    pub fn max_sets(mut self, sets: u32) -> Self {
        self.max_sets = Some(sets);
        self
    }

    pub fn build(self) -> VulkanResult<DescriptorPool> {

        let sizes = self.sizes.expect("Missing sizes for Pool");
        let max_sets = self.max_sets.expect("Missing max sets for Pool");

        #[cfg(debug_assertions)]
        {
           if max_sets == 0 {
                panic!("DescriptorPool max sets cannot be 0");
            }

            if sizes.is_empty() {
                panic!("DescriptorPool must have at least one pool size");
            }
        }

        let create_info = vk::DescriptorPoolCreateInfo::default()
            .flags(self.create_info.flags)
            .pool_sizes(sizes)
            .max_sets(self.create_info.max_sets);
        
        let pool = unsafe {
            profiling::scope!("vkCreateDescriptorPool");
            self.device
                .create_descriptor_pool(&create_info, None)
                .map_err(VulkanError::Unknown)
        }?;

        Ok(DescriptorPool { raw: pool })
    }
}
