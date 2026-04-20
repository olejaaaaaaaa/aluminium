#![doc = include_str!("../README.md")]
pub(crate) mod bindless;
pub(crate) mod camera;
pub(crate) mod core;
pub(crate) mod frame_graph;
pub(crate) mod frame_values;
pub(crate) mod frame_scope;
pub(crate) mod per_frame;
pub(crate) mod render_context;
pub(crate) mod resources;
pub(crate) mod world_renderer;

pub use core::{VulkanError, VulkanResult};

pub use frame_graph::{
    BackBuffer, FrameGraphTexture, FrameGraphTextureDesc, FrameGraphUniform, FrameGraphUniformDesc,
    Handle, RasterPass, RenderTarget, Scissor, Viewport, TransientTexture, TransientBuffer, TemporalStorageBuffer, TemporalStorageTexture,
    TransientStorageBuffer, TransientStorageTexture, LoadOp, StoreOp
};
pub use resources::{
    Mesh, MeshDesc, RasterPipeline, RasterPipelineDesc, Res, Resolution, ShaderType, TextureFormat,
    Transform, TransformDesc, VertexInput,
};
pub use vk_sync::AccessType;
pub use world_renderer::WorldRenderer;
/// Basic types
pub mod types {
    pub use super::core::{PbrVertex, TextureVertex, Vertex};
}
