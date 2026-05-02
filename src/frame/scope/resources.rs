use slotmap::SlotMap;
use crate::{Id, resources::TransientTextureDesc};

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
