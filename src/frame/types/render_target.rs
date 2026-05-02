use super::{ColorAttachment, DepthAttachment};

#[derive(Debug)]
pub struct RenderTarget {
    pub colors: Vec<ColorAttachment>,
    pub depth: Option<DepthAttachment>
}