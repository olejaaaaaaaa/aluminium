use crate::Handle;


pub struct FrameGraphTextureDesc {
    //pub format: TextureFormat,
    //pub resolution: Resolution
}


pub struct RenderTarget {

}

pub struct RenderTargetDesc {
    colors: Vec<Handle<bool>>,
    depth: Option<Handle<bool>>
}

pub struct FrameGraphTexture {}

pub struct BackBuffer {}