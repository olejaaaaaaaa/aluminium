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
    bind: Vec<(u32, u32, vk::DescriptorType, vk::ShaderStageFlags)>,
}

impl<'a> BindlessBuilder<'a> {
    pub fn new(device: &'a Device) -> Self {
        Self {
            device,
            bind: vec![],
        }
    }

    pub fn with(
        mut self,
        bind: u32,
        count: u32,
        _ty: vk::DescriptorType,
        flags: vk::ShaderStageFlags,
    ) -> Self {
        self.bind.push((bind, count, _ty, flags));
        self
    }

    pub fn build(self) -> VulkanResult<Bindless> {
        let mut bindings = vec![];

        for i in self.bind {
            let bind = vk::DescriptorSetLayoutBinding::default()
                .binding(i.0)
                .descriptor_count(i.1)
                .descriptor_type(i.2)
                .stage_flags(i.3);

            bindings.push(bind);
        }

        let set_layout = DescriptorSetLayoutBuilder::new(self.device)
            .bindings((&bindings).to_vec())
            .build()?;

        let mut pool_sizes = vec![];

        for i in bindings {
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
