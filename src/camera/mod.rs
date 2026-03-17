use ash::vk;
use bytemuck::{Pod, Zeroable};

use crate::core::{Device, GpuBuffer, GpuBufferBuilder, VulkanResult};
use crate::ring_buffer::RingBuffer;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct CameraData {
    view: [[f32; 4]; 4],
    proj: [[f32; 4]; 4],
    view_proj: [[f32; 4]; 4],
    inv_view: [[f32; 4]; 4],
    inv_proj: [[f32; 4]; 4],
    inv_view_proj: [[f32; 4]; 4],
}

impl CameraData {
    pub fn identity() -> Self {
        Self {
            view: [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]],
            proj: [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]],
            view_proj: [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]],
            inv_view: [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]],
            inv_proj: [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]],
            inv_view_proj: [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]],
        }
    }
}

pub struct Camera {
    pub is_dirty: bool,
    pub buffer: RingBuffer,
    pub data: CameraData,
}

impl Camera {
    pub fn new(device: &Device, frame_count: usize) -> VulkanResult<Self> {
        let size = size_of::<CameraData>() as u64;

        let buffer = RingBuffer::new(device, size, frame_count, vk::BufferUsageFlags::UNIFORM_BUFFER)?;

        let mut camera = Camera {
            buffer,
            is_dirty: true,
            data: CameraData::identity(),
        };

        camera.buffer.write(device, &[camera.data])?;
        camera.is_dirty = false;

        Ok(camera)
    }

    pub fn buffer(&self) -> vk::Buffer {
        self.buffer.raw
    }

    pub fn begin_frame(&mut self) {
        self.buffer.advance();
    }

    pub fn update(&mut self, device: &Device) -> VulkanResult<()> {
        if self.is_dirty {
            self.buffer.write(device, &[self.data])?;
            self.is_dirty = false;
        }
        Ok(())
    }
}
