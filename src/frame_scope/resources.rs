use std::marker::PhantomData;

use slotmap::{new_key_type, SlotMap};

use crate::FrameGraphTextureDesc;

new_key_type! {
    pub struct Id;
}

pub struct TemporalFrameGraphResources {
    pub textures: SlotMap<Id, FrameGraphTextureDesc>,
}

impl TemporalFrameGraphResources {
    pub fn new() -> Self {
        Self {
            textures: SlotMap::with_key(),
        }
    }
}
