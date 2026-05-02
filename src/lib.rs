#![doc = include_str!("../README.md")]
pub(crate) mod core;
pub(crate) mod ext;
pub(crate) mod frame;
pub(crate) mod render_context;
pub(crate) mod resources;
pub(crate) mod world_renderer;

pub use core::{VulkanError, VulkanResult};

pub use frame::*;
pub use resources::{
    RasterPipeline, RasterPipelineDesc, ShaderType, TextureFormat, TransientTexture,
    VertexInput, Layout, Texture, TextureDesc, Get, GetMut, IndexBuffer, IndexBufferDesc,
    Sampler, StorageBuffer, StorageBufferDesc, Res, VertexBuffer, VertexBufferDesc, Ref, RefMut, PixelFormat
};
pub use world_renderer::WorldRenderer;
/// Basic types
pub mod types {
    pub use super::core::{PbrVertex, TextureVertex, Vertex};
}
