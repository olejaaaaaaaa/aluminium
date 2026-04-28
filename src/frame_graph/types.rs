use crate::{Handle, LoadOp, StoreOp, resources::TransientTexture};

#[derive(Debug)]
pub enum Viewport {
    FullRes,
    HalfRes,
    QuarterRes,
    Custom(u32, u32),
}

#[derive(Debug)]
pub enum Scissor {
    FullRes,
    HalfRes,
    QuarterRes,
    Custom(u32, u32),
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Resolution {
    FullRes,
    HalfRes,
    QuarterRes,
    Custom(u32, u32)
}

#[derive(Debug)]
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
