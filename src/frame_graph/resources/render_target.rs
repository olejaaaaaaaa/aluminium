use crate::frame_graph::types::{ColorAttachment, DepthAttachment};

#[derive(Debug)]
pub struct RenderTarget {
    pub colors: Vec<ColorAttachment>,
    pub depth: Option<DepthAttachment>
}