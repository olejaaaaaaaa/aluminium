use crate::core::{Device, GpuBuffer};

#[derive(Clone, Copy)]
pub struct MeshHandle(pub usize);

pub struct Mesh {
    /// Instance offset
    #[allow(dead_code)]
    pub instance_offset: u32,
    /// Instance count
    #[allow(dead_code)]
    pub instance_count: u32,
    /// Vertex offser
    #[allow(dead_code)]
    pub vertex_offset: u32,
    /// Instance Buffer
    #[allow(dead_code)]
    pub instance_buffer: Option<GpuBuffer>,
    /// Vertex Buffer
    #[allow(dead_code)]
    pub vertex_buffer: GpuBuffer,
    /// Indices
    #[allow(dead_code)]
    pub indices: Option<Vec<u32>>,
    /// Index Buffer
    #[allow(dead_code)]
    pub index_buffer: Option<GpuBuffer>,
}

// Indirect Buffers
//
//

pub struct MeshCollection {
    pub data: Vec<Mesh>,
}

impl MeshCollection {
    #[allow(dead_code)]
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

    #[allow(dead_code)]
    pub fn get_mesh(&self, mesh: MeshHandle) -> &Mesh {
        &self.data[mesh.0]
    }
}
