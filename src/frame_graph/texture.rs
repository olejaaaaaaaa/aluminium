use crate::resources::{Resolution, TextureFormat};
use crate::Handle;

pub struct FrameGraphTextureDesc {
    pub format: TextureFormat,
    pub resolution: Resolution,
}

pub struct RenderTarget<'a> {
    pub colors: &'a [Handle<FrameGraphTexture>],
    pub depth: Option<Handle<FrameGraphTexture>>,
}

pub struct FrameGraphTexture {}

pub struct BackBuffer {}
