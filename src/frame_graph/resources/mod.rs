

mod buffers;
use std::sync::Arc;

pub use buffers::*;

mod textures;
use slotmap::SecondaryMap;
pub use textures::*;

mod render_target;
pub use render_target::*;

mod handle;
pub use handle::Handle;

use crate::{core::ImageView, frame_scope::{FrameScope, Id, TransientTextureDesc}, render_context::RenderContext};

pub struct FrameGraphResources {
    pub backbuffers: SecondaryMap<Id, ash::vk::ImageView>,
    pub transient_textures: SecondaryMap<Id, ImageView>
}

pub struct FrameSlot {
    desc: TransientTextureDesc,
    index: usize
}

impl FrameGraphResources {
    pub fn new() -> Self {
        Self {
            backbuffers: SecondaryMap::new(),
            transient_textures: SecondaryMap::new()
        }
    }

    pub fn prepare_backbuffers(&mut self, index: u32, scope: &FrameScope<'_>, ctx: &Arc<RenderContext>) {
        let image_views = &ctx.window.read().image_views;
        let image_view = image_views[index as usize].raw;
        for i in scope.resources.backbuffers.keys() {
            self.backbuffers.insert(i, image_view);
        }
    }
}
