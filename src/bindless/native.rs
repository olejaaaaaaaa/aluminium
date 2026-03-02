use ash::vk;

use crate::core::{
    DescriptorPool, DescriptorPoolBuilder, DescriptorSetLayout, DescriptorSetLayoutBuilder, Device,
    VulkanResult,
};
use crate::render_context::{Feature, RenderContext};




pub(crate) struct NativeBindless {
    pub(crate) set_layout: DescriptorSetLayout,
    pub(crate) set: vk::DescriptorSet,
    pub(crate) pool: DescriptorPool,
}

impl NativeBindless {
    pub fn new(
        device: &Device,
        layouts: &[vk::DescriptorSetLayoutBinding<'static>],
    ) -> VulkanResult<Self> {

        let binding_flags: Vec<vk::DescriptorBindingFlags> = layouts
            .iter()
            .map(|_| {
                vk::DescriptorBindingFlags::UPDATE_AFTER_BIND
                    | vk::DescriptorBindingFlags::PARTIALLY_BOUND
                    | vk::DescriptorBindingFlags::UPDATE_UNUSED_WHILE_PENDING
            })
            .collect();

        let mut binding_flags_info = vk::DescriptorSetLayoutBindingFlagsCreateInfo::default()
            .binding_flags(&binding_flags);

        let set_layout = DescriptorSetLayoutBuilder::new(device)
            .bindings(layouts.to_vec())
            .flags(vk::DescriptorSetLayoutCreateFlags::UPDATE_AFTER_BIND_POOL) 
            .push_next(&mut binding_flags_info)                             
            .build()?;

        let mut pool_sizes = vec![];

        for i in layouts {
            pool_sizes.push(
                vk::DescriptorPoolSize::default()
                    .descriptor_count(i.descriptor_count)
                    .ty(i.descriptor_type),
            );
        }

        let pool = DescriptorPoolBuilder::new(device)
            .flags(vk::DescriptorPoolCreateFlags::UPDATE_AFTER_BIND)
            .pool_sizes(&pool_sizes)
            .max_sets(1)
            .build()?;

        let layouts = [set_layout.raw];
        let set = pool.create_descriptor_set(device, &layouts)[0];

        Ok(Self {
            set_layout,
            set,
            pool,
        })
    }

    pub fn destroy(&self, device: &Device) {
        unsafe {
            device.destroy_descriptor_pool(self.pool.raw, None);
            device.destroy_descriptor_set_layout(self.set_layout.raw, None);
        }
    }

    pub fn update_buffer_set(
        &self,
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
