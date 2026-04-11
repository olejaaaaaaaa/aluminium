use crate::{Res, core::{Image, ImageView}, resources::{Destroy, ResourceKey}};
mod texture;
pub use texture::{Resolution, TextureFormat};
const MAX_TEXTURE: usize = 100000;

#[derive(Clone, Copy)]
pub struct TextureHandle(usize);

pub struct Texture {
    image: Image,
}

pub struct TextureDesc<'a> {
    width: u32,
    height: u32,
    format: TextureFormat,
    pixels: &'a [u8]
}

pub struct TextureView {
    image: Res<Image>,
    view: Res<ImageView>
}

pub struct TextureViewDesc {
    image: Res<Image>
}

pub struct TexturePool {
    
}

impl TexturePool {
    fn new() -> Self {
        Self { 

        }
    }
}

impl Destroy for Image {
    fn destroy(handle: ResourceKey, ctx: std::sync::Weak<crate::render_context::RenderContext>, resources: std::sync::Weak<super::Resources>) {
        
    }
}

impl Destroy for ImageView {
    fn destroy(handle: ResourceKey, ctx: std::sync::Weak<crate::render_context::RenderContext>, resources: std::sync::Weak<super::Resources>) {
        
    }
}