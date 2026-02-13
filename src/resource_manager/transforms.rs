use ash::vk;
use bytemuck::{Pod, Zeroable};

use crate::core::{Device, GpuBuffer, GpuBufferBuilder, VulkanResult};

const MAX_TRANSFORMS: usize = 10000;

#[derive(Clone, Copy)]
pub struct TransformHandle(pub(crate) usize);

/// Transform for Mesh
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Transform {
    /// Rot
    pub rot: [[f32; 4]; 4],
    /// Scale
    pub scale: [f32; 4],
    /// Pos
    pub pos: [f32; 4],
}

impl Transform {
    /// identity matrix
    pub fn identity() -> Self {
        Self {
            scale: [1.0, 1.0, 1.0, 0.0],
            rot: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
            pos: [0.0, 0.0, 0.0, 0.0],
        }
    }
}

pub struct TransformCollection {
    pub is_dirty: bool,
    pub data: Vec<Transform>,
    pub buffer: GpuBuffer,
}

impl TransformCollection {
    #[allow(dead_code)]
    pub fn destroy(&mut self, device: &Device) {
        unsafe {
            device
                .allocator
                .destroy_buffer(self.buffer.raw, &mut self.buffer.allocation);
        }
    }

    pub fn new(device: &Device) -> VulkanResult<Self> {
        let count = MAX_TRANSFORMS;
        let size = (size_of::<Transform>() * count) as u64;

        let mut buffer = GpuBufferBuilder::cpu_only(device)
            .usage(vk::BufferUsageFlags::STORAGE_BUFFER)
            .size(size)
            .build()?;

        let data = vec![Transform::identity(); count];

        buffer.upload_data(device, &data)?;

        Ok(Self {
            data,
            buffer,
            is_dirty: false,
        })
    }

    #[allow(dead_code)]
    pub fn get_mut(&mut self, handle: &TransformHandle) -> &mut Transform {
        self.is_dirty = true;
        &mut self.data[handle.0]
    }

    #[allow(dead_code)]
    pub fn get(&self, handle: &TransformHandle) -> &Transform {
        &self.data[handle.0]
    }

    pub fn create_transform(&mut self, data: Transform) -> TransformHandle {
        self.is_dirty = true;
        let index = self.data.len();
        self.data.push(data);
        TransformHandle(index)
    }

    #[allow(dead_code)]
    pub fn update(&mut self, device: &Device) -> VulkanResult<()> {
        if self.is_dirty {
            self.is_dirty = false;
            return self.buffer.upload_data(device, &self.data);
        }
        Ok(())
    }
}
