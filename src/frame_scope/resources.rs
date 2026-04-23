use std::marker::PhantomData;

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

pub struct TemporalFrameGraphResources {
    pub textures: SlotMap<Id, TransientTextureDesc>,
}

impl TemporalFrameGraphResources {
    pub fn new() -> Self {
        Self {
            textures: SlotMap::with_key(),
        }
    }
}
