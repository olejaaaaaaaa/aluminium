use std::sync::Weak;

use ash::vk;
use bytemuck::{Pod, Zeroable};
use log::warn;

use crate::core::{Device, GpuBuffer, GpuBufferBuilder};
use crate::resources::{self, Create, Destroy, Pool, Resources};
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
}

impl<'a> MeshDesc<'a> {
    pub fn new<T: Pod + Zeroable>(vertices: &'a [T]) -> MeshDesc<'a> {
        MeshDesc {
            vertices: bytemuck::cast_slice(vertices),
        }
    }
}

impl Destroy for Mesh {
    fn destroy(key: super::ResourceKey, resources: &super::Resources) {
        warn!("Mesh leaking with key {:?}", key);
    }
}

// impl Create for Mesh {
//     type Desc<'a> = MeshDesc<'a>;
//     fn create(resources: &super::Resources, desc: Self::Desc<'_>) -> VulkanResult<super::Res<Self>> {
//         let ctx = &resources.ctx;
//         let size = std::mem::size_of_val(desc.vertices) as u64;

//         let mut vertex_buffer = GpuBufferBuilder::cpu_only(&ctx.device)
//             .size(size)
//             .usage(vk::BufferUsageFlags::VERTEX_BUFFER)
//             .build()?;

//         vertex_buffer.upload_data(&ctx.device, desc.vertices)?;

//         let mesh = resources.mesh.write().data.insert(Mesh {
//             instance_offset: 0,
//             instance_count: 1,
//             vertex_offset: 0,
//             vertex_buffer,
//             index_buffer: None,
//         });

//         Ok(mesh)
//     }
// }

pub struct MeshStore {
    pub data: Pool<Mesh>,
}

impl MeshStore {
    pub fn new(resources: Weak<Resources>) -> Self {
        Self { data: Pool::new(resources) }
    }
}
