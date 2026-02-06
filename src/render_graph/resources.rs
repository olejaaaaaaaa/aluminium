use ash::vk::{self, DescriptorSet};
use slotmap::{new_key_type, SecondaryMap, SlotMap};

use super::PassContext;
use crate::core::{
    CommandPool, CommandPoolBuilder, DescriptorSetLayout, Device, FrameBuffer, FrameBufferBuilder,
    GraphicsPipeline, Image, ImageBuilder, ImageView, ImageViewBuilder, PipelineLayout, Sampler,
    SamplerBuilder, VulkanResult,
};
use crate::render_graph::{LoadOp, StoreOp, TextureDesc};

new_key_type! {
    pub struct DescriptorSetHandle;
}

new_key_type! {
    pub struct TextureHandle;
}

#[derive(Debug, Clone, Copy)]
pub enum RenderGraphResource {
    Texture {
        handle: TextureHandle,
        last_access: vk_sync::AccessType,
    },
    RenderTarget {
        texture: (TextureHandle, vk_sync::AccessType),
        ops: (LoadOp, StoreOp),
    },
}

impl RenderGraphResource {
    pub fn last_access(&self) -> vk_sync::AccessType {
        match self {
            RenderGraphResource::Texture {
                handle,
                last_access,
            } => *last_access,
            RenderGraphResource::RenderTarget { texture, ops } => texture.1,
        }
    }
}

impl Into<RenderGraphResource> for TextureHandle {
    fn into(self) -> RenderGraphResource {
        RenderGraphResource::Texture {
            handle: self,
            last_access: vk_sync::AccessType::Nothing,
        }
    }
}

pub struct RenderGraphResources {
    pub(crate) textures: SlotMap<TextureHandle, TextureDesc>,
}

impl RenderGraphResources {
    pub fn new() -> Self {
        RenderGraphResources {
            textures: SlotMap::with_key(),
        }
    }

    pub fn registry_texture(&mut self, desc: TextureDesc) -> TextureHandle {
        self.textures.insert(desc)
    }

    pub fn get_texture(&mut self, handle: TextureHandle) -> Option<&TextureDesc> {
        self.textures.get(handle)
    }
}
