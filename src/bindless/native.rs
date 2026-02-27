
use ash::vk;

use crate::core::{
    DescriptorPool, DescriptorPoolBuilder, DescriptorSetLayout, DescriptorSetLayoutBuilder, Device,
    VulkanResult,
};
use crate::render_context::{Feature, RenderContext};

pub struct NativeBindless {
    descriptor_set: vk::DescriptorSet 
}


