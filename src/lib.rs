//! Lightweight abstraction for rendering using Vulkan API
pub(crate) mod bindless;
// pub(crate) mod bvh;
pub(crate) mod camera;
pub(crate) mod core;
pub(crate) mod reflection;
pub(crate) mod render_context;
pub(crate) mod render_graph;
pub(crate) mod resource_manager;
pub(crate) mod world_renderer;

pub use render_graph::{
    ComputePass, LoadOp, PresentPass, RasterPass, Resolution, SamplerType, StoreOp, TextureDesc,
    TextureFormat, TextureUsage,
};
pub use resource_manager::{Material, Renderable, Transform};
pub use world_renderer::WorldRenderer;

/// Basic types
pub mod types {
    pub use super::core::{PBRVertex, TextureVertex, Vertex};
}
