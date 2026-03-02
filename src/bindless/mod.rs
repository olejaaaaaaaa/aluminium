use std::sync::Arc;

use ash::vk;

mod native;
use native::*;

mod software;
use software::*;

use crate::core::{Device, Extension, VulkanResult};
use crate::render_context::{RenderContext};

/// Abstraction over Bindless technology, even if the device does not natively support it
#[derive(Clone)]
pub enum Bindless {
    // Only one Descriptor Set
    // A real native bindless
    Native(Arc<NativeBindless>),
    // Per Frame Descriptor Set
    // Android most likely does not have native bindless support
    Software(Arc<SoftwareBindless>),
}

impl Bindless {
    pub fn new(
        ctx: &RenderContext,
        layouts: &[vk::DescriptorSetLayoutBinding<'static>],
    ) -> VulkanResult<Self> {
        if ctx.check_features(&[Extension::Bindless]) {
            Ok(Bindless::Native(Arc::new(NativeBindless::new(
                &ctx.device, 
                layouts
            )?)))
        } else {
            Ok(Bindless::Software(Arc::new(SoftwareBindless::new(
                &ctx.device,
                layouts,
            )?)))
        }
    }

    pub fn bindless_set_layout(&self) -> vk::DescriptorSetLayout {
        match self {
            Bindless::Software(software) => software.set_layout.raw,
            Bindless::Native(native) => native.set_layout.raw
        }
    }

    pub fn bindless_set(&self) -> vk::DescriptorSet {
        match self {
            Bindless::Software(software) => software.set,
            Bindless::Native(native) => native.set
        }
    }

    pub fn update_buffer_set(
        &self,
        device: &Device,
        bind: u32,
        ty: vk::DescriptorType,
        buffer: vk::Buffer,
        offset: vk::DeviceSize,
        range: vk::DeviceSize,
    ) {
        match self {
            Bindless::Native(native) => {
                native.update_buffer_set(device, bind, ty, buffer, offset, range);
            },
            Bindless::Software(software) => {
                software.update_buffer_set(device, bind, ty, buffer, offset, range);
            },
        }
    }
}
