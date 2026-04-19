use crate::resources::{Resolution, TextureFormat};
use crate::Handle;

#[derive(Debug)]
pub struct FrameGraphTextureDesc {
    pub name: &'static str,
    pub format: TextureFormat,
    pub resolution: Resolution,
}

pub struct RenderTarget {
    pub colors: Vec<Handle<FrameGraphTexture>>,
    pub depth: Option<Handle<FrameGraphTexture>>,
}

pub struct FrameGraphUniform {

}

pub struct FrameGraphUniformDesc {

}

impl FrameGraphUniformDesc {
    pub fn new() -> Self {
        Self {  }
    }

    pub fn with<T>(mut self, handle: Handle<T>) -> Self {
        self
    }
}

pub struct FrameGraphTexture {

}

pub struct BackBuffer {

}
