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
    Get, GetMut, IndexBuffer, IndexBufferDesc, Layout, PixelFormat, RasterPipeline,
    RasterPipelineDesc, Ref, RefMut, Res, Sampler, ShaderType, StorageBuffer, StorageBufferDesc,
    Texture, TextureDesc, TextureFormat, TransientTexture, VertexBuffer, VertexBufferDesc,
    VertexInput, Uniform, UniformBinding, UniformType, ShaderStage
};
pub use world_renderer::WorldRenderer;
/// Basic types
pub mod types {
    pub use super::core::{PbrVertex, TextureVertex, Vertex};
}
