use std::sync::Arc;

use ash::vk;

mod native;
use native::*;

mod software;
use software::*;

use crate::core::{Device, Extension, VulkanResult};
use crate::render_context::{Feature, RenderContext};

/// Abstraction for bindless rendering
/// The GPU may not support this natively
#[derive(Clone)]
pub enum Bindless {
    Native(Arc<NativeBindless>),
    Software(Arc<SoftwareBindless>),
}

impl Bindless {
    pub fn new(
        ctx: &RenderContext,
        layouts: &[vk::DescriptorSetLayoutBinding<'static>],
    ) -> VulkanResult<Self> {
        if !ctx.check_features(&[Extension::Bindless]) {
            todo!()
        } else {
            Ok(Bindless::Software(Arc::new(SoftwareBindless::new(
                &ctx.device,
                layouts,
            )?)))
        }
    }

    pub fn bindless_set(&self) -> vk::DescriptorSet {
        match self {
            Bindless::Software(bindless) => bindless.set,
            Bindless::Native(_) => {
                todo!()
            },
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
            Bindless::Native(_) => {
                todo!()
            },
            Bindless::Software(bindless) => {
                bindless.update_buffer_set(device, bind, ty, buffer, offset, range);
            },
        }
    }
}
