use crate::Handle;


pub struct FrameGraphTextureDesc {
    //pub format: TextureFormat,
    //pub resolution: Resolution
}


pub struct RenderTarget {

}

pub struct RenderTargetsDesc<'a> {
    pub colors: &'a [Handle<bool>],
    pub depth: Option<Handle<bool>>
}

pub struct FrameGraphTexture {}

pub struct BackBuffer {}