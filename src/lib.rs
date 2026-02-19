//! Lightweight abstraction for rendering using Vulkan API
//!
//! # Cargo Features
//! - vma (AMD Vulkan Memory Allocator used by default)
//! - gpu-allocator (Traverse Research Gpu Allocator without С/C++ dependencies)
//! - runtime-check (Runtime check used by default)
pub(crate) mod bindless;
// TODO:
// pub(crate) mod bvh;
pub(crate) mod buffering;
pub(crate) mod camera;
pub(crate) mod core;
pub(crate) mod frame_values;
pub(crate) mod reflection;
pub(crate) mod render_context;
pub(crate) mod render_graph;
pub(crate) mod resource_manager;
pub(crate) mod world_renderer;

pub use render_graph::{
    ComputePass, LoadOp, PresentPass, PresentPassBuilder, RasterPass, RasterPassBuilder,
    Resolution, SamplerType, StoreOp, TextureDesc, TextureFormat, TextureUsage,
};
pub use resource_manager::{Material, Renderable, Transform};
pub use world_renderer::WorldRenderer;

/// Basic types
pub mod types {
    pub use super::core::{PBRVertex, TextureVertex, Vertex};
}
