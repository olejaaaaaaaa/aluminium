use crate::core::{Device, GpuBuffer, PBRVertex, Vertex};

const MAX_MESH: usize = 100000;

#[derive(Clone, Copy)]
pub struct MeshHandle(pub usize);

pub struct Mesh {
    pub instance_offset: u32,
    pub instance_count: u32,
    pub vertex_offset: u32,
    pub data: Option<MeshData>,
    pub instance_buffer: Option<GpuBuffer>,
    pub vertex_buffer: GpuBuffer,
    pub indices: Option<Vec<u32>>,
    pub index_buffer: Option<GpuBuffer>,
}

pub struct MeshCollection {
    pub data: Vec<Mesh>,
}

impl MeshCollection {
    pub fn destroy(&mut self, device: &Device) {
        unsafe {
            for i in &mut self.data {
                device
                    .allocator
                    .destroy_buffer(i.vertex_buffer.raw, &mut i.vertex_buffer.allocation);
                if let Some(index_buffer) = &mut i.index_buffer {
                    device
                        .allocator
                        .destroy_buffer(index_buffer.raw, &mut index_buffer.allocation);
                }
            }
        }
    }

    pub fn new() -> Self {
        Self { data: vec![] }
    }

    pub fn add_mesh(&mut self, mesh: Mesh) -> MeshHandle {
        let index = self.data.len();
        self.data.push(mesh);
        MeshHandle(index)
    }

    pub fn get_mesh(&self, mesh: MeshHandle) -> &Mesh {
        &self.data[mesh.0]
    }
}

pub enum MeshData {
    Simple(Vec<Vertex>),
    PBR(Vec<PBRVertex>),
}
