use slotmap::{new_key_type, SlotMap};

use crate::render_graph::TextureDesc;

new_key_type! {
    pub struct DescriptorSetHandle;
    pub struct TextureHandle;
}

#[derive(Debug, Clone, Copy)]
pub enum RenderGraphResource {
    Texture {
        handle: TextureHandle,
        last_access: vk_sync::AccessType,
    },
}

impl RenderGraphResource {
    pub fn last_access(&self) -> vk_sync::AccessType {
        match self {
            RenderGraphResource::Texture {
                handle: _,
                last_access,
            } => *last_access,
            // RenderGraphResource::RenderTarget { texture, ops: _ } => texture.1,
        }
    }
}

impl From<TextureHandle> for RenderGraphResource {
    fn from(val: TextureHandle) -> Self {
        RenderGraphResource::Texture {
            handle: val,
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
