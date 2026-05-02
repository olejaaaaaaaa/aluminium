use super::{LoadOp, StoreOp, Handle};
use crate::TransientTexture;

#[derive(Debug)]
pub struct ColorAttachment {
    pub color: Handle<TransientTexture>,
    pub load: LoadOp,
    pub store: StoreOp
}

#[derive(Debug)]
pub struct DepthAttachment {
    pub depth: Handle<TransientTexture>,
    pub load: LoadOp,
    pub store: StoreOp
}
