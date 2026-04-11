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
    fn destroy(key: ResourceKey, _ctx: Weak<crate::render_context::RenderContext>, _resources: Weak<Resources>) {}
}

impl Create for Mesh {
    type Desc<'a> = MeshDesc<'a>;
    fn create(ctx: &Arc<RenderContext>, resources: &Arc<super::Resources>, desc: Self::Desc<'_>) -> VulkanResult<super::Res<Self>> {
        let size = std::mem::size_of_val(desc.vertices) as u64;

        let mut vertex_buffer = GpuBufferBuilder::cpu_only(&ctx.device)
            .size(size)
            .usage(vk::BufferUsageFlags::VERTEX_BUFFER)
            .build()?;

        vertex_buffer.upload_data(desc.vertices)?;

        let index_buffer = if let Some(indices) = desc.indices {
            let mut index_buffer = GpuBufferBuilder::cpu_only(&ctx.device)
                .size(size_of_val(indices) as u64)
                .usage(vk::BufferUsageFlags::INDEX_BUFFER)
                .build()?;

            index_buffer.upload_data(indices)?;

            Some(index_buffer)
        } else {
            None
        };

        let key = resources.meshes.write().insert(
            Mesh {
                instance_offset: 0,
                instance_count: 1,
                vertex_offset: 0,
                vertex_buffer,
                index_buffer,
            },
        );

        Ok(resources.make_handle(ctx, key))
    }
}

pub struct MeshStore {
    pub data: Pool<Mesh>,
}

impl MeshStore {
    pub fn new() -> Self {
        Self { data: Pool::new() }
    }

    pub fn destroy(&mut self, device: &Device) {

    }
}
