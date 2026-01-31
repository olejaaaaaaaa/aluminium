use super::{LoadOp, StoreOp};
use crate::render_graph::TextureHandle;

pub struct RtPass {}

impl RtPass {
    pub fn new() {}

    pub fn write_texture(&mut self, texture: TextureHandle, load: LoadOp, store: StoreOp) {}

    pub fn read() {}

    pub fn write() {}

    pub fn trace_rays() {}
}
