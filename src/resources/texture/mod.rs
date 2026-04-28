use crate::core::{Image, ImageView};
use crate::resources::{Destroy, ResourceKey};
use crate::{Res, Resolution};
const MAX_TEXTURE: usize = 100000;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum TextureFormat {
    Depth,
    DepthStencil,
    HDR,
    Color,
    Data
}

#[derive(Clone, Copy)]
pub struct TextureHandle(usize);

pub struct Texture {
    image: Image,
}

pub struct TextureDesc<'a> {
    width: u32,
    height: u32,
    format: TextureFormat,
    pixels: &'a [u8],
}

pub struct TextureView {
    image: Res<Image>,
    view: Res<ImageView>,
}

pub struct TextureViewDesc {
    image: Res<Image>,
}

pub struct TexturePool {}

impl TexturePool {
    fn new() -> Self {
        Self {}
    }
}

impl Destroy for Image {
    fn destroy(
        handle: ResourceKey,
        ctx: std::sync::Weak<crate::render_context::RenderContext>,
        resources: std::sync::Weak<super::Resources>,
    ) {
    }
}

impl Destroy for ImageView {
    fn destroy(
        handle: ResourceKey,
        ctx: std::sync::Weak<crate::render_context::RenderContext>,
        resources: std::sync::Weak<super::Resources>,
    ) {
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct TransientTextureDesc {
    pub name: &'static str,
    pub format: TextureFormat,
    pub resolution: Resolution,
}

pub struct TransientTexture {
    pub(crate) image: Image,
    pub(crate) view: ImageView
}

impl Destroy for TransientTexture {
    fn destroy(key: ResourceKey, ctx: std::sync::Weak<crate::render_context::RenderContext>, resources: std::sync::Weak<super::Resources>) {
        
    }
}

impl PartialEq for TransientTexture {
    fn eq(&self, other: &Self) -> bool {
        self.image.raw == other.image.raw && self.view.raw == other.view.raw
    }
}