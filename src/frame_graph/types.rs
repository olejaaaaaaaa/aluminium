use crate::{Handle, LoadOp, StoreOp, TransientTexture};

pub enum Viewport {
    FullRes,
    HalfRes,
    QuarterRes,
    Custom(u32, u32),
}

pub enum Scissor {
    FullRes,
    HalfRes,
    QuarterRes,
    Custom(u32, u32),
}

pub struct Location {
    pub set: u32,
    pub binding: u32
}

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
