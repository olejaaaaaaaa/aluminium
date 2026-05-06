use super::{Handle, LoadOp, StoreOp};
use crate::TransientTexture;

pub enum Color {
    Internal(Handle<TransientTexture>),
    External,
}

#[derive(Debug)]
pub struct ColorAttachment {
    pub color: Handle<TransientTexture>,
    pub load: LoadOp,
    pub store: StoreOp,
}

#[derive(Debug)]
pub struct DepthAttachment {
    pub depth: Handle<TransientTexture>,
    pub load: LoadOp,
    pub store: StoreOp,
}
