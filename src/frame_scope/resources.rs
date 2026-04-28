use std::{marker::PhantomData, sync::Arc};

use slotmap::{new_key_type, SlotMap};

use crate::{Resolution, TextureFormat, resources::TransientTextureDesc};

new_key_type! {
    pub struct Id;
}

#[derive(Debug)]
pub struct FrameResources {
    pub textures: SlotMap<Id, TransientTextureDesc>,
}

impl FrameResources {
    pub fn new() -> Self {
        Self {
            textures: SlotMap::with_key(),
        }
    }
}
