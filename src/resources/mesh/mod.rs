use std::sync::{Arc, Weak};

use ash::vk;
use bytemuck::{Pod, Zeroable};

use crate::core::{Device, GpuBuffer, GpuBufferBuilder};
use crate::render_context::RenderContext;
use crate::resources::{Create, Destroy, Pool, ResourceKey, Resources};
use crate::VulkanResult;

pub struct Mesh {
    /// Instance offset
    pub instance_offset: u32,
    /// Instance count
    pub instance_count: u32,
    /// Vertex offser
    pub vertex_offset: u32,
    /// Vertex Buffer
    pub vertex_buffer: GpuBuffer,
    /// Index Buffer
    pub index_buffer: Option<GpuBuffer>,
}

pub enum IndexType {
    U32,
    U16
}

pub struct IndexBuffer {
    pub(crate) buffer: GpuBuffer,
    pub(crate) ty: IndexType
}

pub struct IndexBufferDesc<'a> {
    indices: &'a [u32],
}

impl<'a> IndexBufferDesc<'a> {
    pub fn new(indices: &'a [u32]) -> Self {
        Self { indices }
    }
}

pub struct VertexBuffer {
    pub(crate) buffer: GpuBuffer
}

pub struct UniformBuffer {
    buffers: Vec<GpuBuffer>,
    data: bool
}

pub struct StorageBuffer {

}

pub struct VertexBufferDesc<'a> {
    vertices: &'a [u8],
}

impl<'a> VertexBufferDesc<'a> {
    pub fn new<T: Pod + Zeroable>(vertices: &'a [T]) -> Self {
        Self { vertices: bytemuck::cast_slice(vertices) }
    }
}

impl Destroy for VertexBuffer {
    fn destroy(
        _key: ResourceKey,
        _ctx: Weak<crate::render_context::RenderContext>,
        _resources: Weak<Resources>,
    ) {
    }
}

impl Create for VertexBuffer {
    type Desc<'a> = VertexBufferDesc<'a>;
    fn create(
        ctx: &Arc<RenderContext>,
        resources: &Arc<super::Resources>,
        desc: Self::Desc<'_>,
    ) -> VulkanResult<super::Res<Self>> {

        let size = std::mem::size_of_val(desc.vertices) as u64;

        let mut vertex_buffer = GpuBufferBuilder::cpu_only(&ctx.device)
            .size(size)
            .usage(vk::BufferUsageFlags::VERTEX_BUFFER)
            .build()?;

        vertex_buffer.upload_data(desc.vertices)?;

        let key = resources.vertices.write().insert(VertexBuffer {
            buffer: vertex_buffer,
        });

        Ok(resources.make_handle(ctx, key))
    }
}

impl Create for IndexBuffer {
    type Desc<'a> = IndexBufferDesc<'a>;
    fn create(
        ctx: &Arc<RenderContext>,
        resources: &Arc<super::Resources>,
        desc: Self::Desc<'_>,
    ) -> VulkanResult<super::Res<Self>> {

        let size = std::mem::size_of_val(desc.indices) as u64;

        let mut index_buffer = GpuBufferBuilder::cpu_only(&ctx.device)
            .size(size)
            .usage(vk::BufferUsageFlags::INDEX_BUFFER)
            .build()?;

        index_buffer.upload_data(desc.indices)?;

        let key = resources.indices.write().insert(IndexBuffer {
            buffer: index_buffer,
            ty: IndexType::U32
        });

        Ok(resources.make_handle(ctx, key))
    }
}

impl Destroy for IndexBuffer {
    fn destroy(
        _key: ResourceKey,
        _ctx: Weak<crate::render_context::RenderContext>,
        _resources: Weak<Resources>,
    ) {
    }
}

pub struct MeshDesc<'a> {
    vertices: &'a [u8],
    indices: Option<&'a [u32]>,
}

impl<'a> MeshDesc<'a> {
    pub fn new<T: Pod + Zeroable>(vertices: &'a [T]) -> MeshDesc<'a> {
        MeshDesc {
            vertices: bytemuck::cast_slice(vertices),
            indices: None,
        }
    }

    pub fn with_indices(mut self, indices: &'a [u32]) -> MeshDesc<'a> {
        self.indices = Some(indices);
        self
    }
}

impl Destroy for Mesh {
    fn destroy(
        key: ResourceKey,
        _ctx: Weak<crate::render_context::RenderContext>,
        _resources: Weak<Resources>,
    ) {
    }
}
