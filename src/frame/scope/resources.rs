use slotmap::SlotMap;

use crate::resources::TransientTextureDesc;
use crate::Id;

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
