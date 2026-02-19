use ash::vk;

use crate::core::{
    DescriptorPool, DescriptorPoolBuilder, DescriptorSetLayout, DescriptorSetLayoutBuilder, Device,
    VulkanResult,
};

/// Abstraction for bindless rendering
/// The GPU may not support this natively
pub(crate) struct Bindless {
    pub(crate) set_layout: DescriptorSetLayout,
    pub(crate) set: vk::DescriptorSet,
    pub(crate) pool: DescriptorPool,
}

pub struct BindlessBuilder<'a> {
    device: &'a Device,
    layouts: &'a [vk::DescriptorSetLayoutBinding<'static>],
}

impl<'a> BindlessBuilder<'a> {
    pub fn new(device: &'a Device, layouts: &'a [vk::DescriptorSetLayoutBinding<'static>]) -> Self {
        Self { device, layouts }
    }

    pub fn build(self) -> VulkanResult<Bindless> {
        let set_layout = DescriptorSetLayoutBuilder::new(self.device)
            .bindings(self.layouts.to_vec())
            .build()?;

        let mut pool_sizes = vec![];

        for i in self.layouts {
            pool_sizes.push(
                vk::DescriptorPoolSize::default()
                    .descriptor_count(i.descriptor_count)
                    .ty(i.descriptor_type),
            );
        }

        let pool = DescriptorPoolBuilder::new(self.device)
            .pool_sizes(&pool_sizes)
            .max_sets(1)
            .build()?;

        let layouts = [set_layout.raw];
        let set = pool.create_descriptor_set(self.device, &layouts)[0];

        Ok(Bindless {
            set_layout,
            set,
            pool,
        })
    }
}

impl Bindless {
    pub fn destroy(&self, device: &Device) {
        unsafe {
            device.destroy_descriptor_pool(self.pool.raw, None);
            device.destroy_descriptor_set_layout(self.set_layout.raw, None);
        }
    }

    pub fn update_buffer_set(
        &mut self,
        device: &Device,
        bind: u32,
        _ty: vk::DescriptorType,
        buffer: vk::Buffer,
        offset: vk::DeviceSize,
        range: vk::DeviceSize,
    ) {
        let buffer_info = vk::DescriptorBufferInfo::default()
            .buffer(buffer)
            .offset(offset)
            .range(range);

        let write = vk::WriteDescriptorSet::default()
            .dst_set(self.set)
            .dst_binding(bind)
            .dst_array_element(0)
            .descriptor_type(_ty)
            .buffer_info(std::slice::from_ref(&buffer_info));

        unsafe { device.update_descriptor_sets(&[write], &[]) };
    }
}
