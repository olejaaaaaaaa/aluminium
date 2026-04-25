use std::{marker::PhantomData, sync::Arc};

use slotmap::{new_key_type, SlotMap};

use crate::{Resolution, TextureFormat};

new_key_type! {
    pub struct Id;
}

#[derive(Debug)]
pub struct TransientTextureDesc {
    pub name: &'static str,
    pub format: TextureFormat,
    pub resolution: Resolution,
}

pub struct FrameResources {
    pub backbuffers: SlotMap<Id, TransientTextureDesc>,
    pub textures: SlotMap<Id, TransientTextureDesc>,
}

impl FrameResources {
    pub fn new() -> Self {
        Self {
            backbuffers: SlotMap::with_key(),
            textures: SlotMap::with_key(),
        }
    }
}
