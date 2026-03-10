//! Lightweight abstraction for rendering using Vulkan API
//!
//! # Cargo Features
//! - vma (AMD Vulkan Memory Allocator used by default)
//! - gpu-allocator (Traverse Research Gpu Allocator without С/C++ dependencies)
//! - runtime-check (Runtime check used by default)
pub(crate) mod acceleration;
pub(crate) mod bindless;
pub(crate) mod camera;
pub(crate) mod core;
pub(crate) mod frame_values;
pub(crate) mod per_frame;
pub(crate) mod reflection;
pub(crate) mod render_context;
pub(crate) mod render_graph;
pub(crate) mod resource_manager;
pub(crate) mod ring_buffer;
pub(crate) mod world_renderer;

pub use core::{VulkanError, VulkanResult};

pub use render_graph::{
    ComputePass, LoadOp, PresentPass, PresentPassBuilder, RasterPass, RasterPassBuilder,
    Resolution, SamplerType, ShaderStage, ShaderType, StoreOp, TextureDesc, TextureFormat,
    TextureUsage, UniformBinding, VertexInput,
};
pub use resource_manager::{
    Material, MaterialHandle, MeshHandle, Renderable, RenderableHandle, Transform, TransformHandle,
    UniformValue,
};
pub use world_renderer::WorldRenderer;

/// Basic types
pub mod types {
    pub use super::core::{PBRVertex, TextureVertex, Vertex};
}
