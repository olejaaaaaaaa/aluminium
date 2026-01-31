use std::f32::consts::{FRAC_PI_2, PI};

use ash::vk;
use bytemuck::{Pod, Zeroable};

use crate::core::{Device, GpuBuffer, GpuBufferBuilder, VulkanResult};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct CameraData {
    view: [[f32; 4]; 4],
    proj: [[f32; 4]; 4],
    pos: [f32; 4],
}

impl CameraData {
    pub fn identity() -> Self {
        Self {
            view: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
            proj: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
            pos: [0.0, 0.0, 0.0, 0.0],
        }
    }
}

pub struct Camera {
    pub is_dirty: bool,
    pub buffer: GpuBuffer,
    pub data: CameraData,
}

impl Camera {
    pub fn destroy(&mut self, device: &Device) {
        unsafe {
            device
                .allocator
                .destroy_buffer(self.buffer.raw, &mut self.buffer.allocation);
        }
    }

    pub fn new(device: &Device) -> VulkanResult<Self> {
        let size = size_of::<CameraData>() as u64;
        let buffer = GpuBufferBuilder::cpu_only(device)
            .size(size)
            .usage(vk::BufferUsageFlags::UNIFORM_BUFFER)
            .build()?;

        let mut camera = Camera {
            is_dirty: true,
            buffer,
            data: CameraData::identity(),
        };

        camera.buffer.upload_data(device, &[camera.data])?;
        camera.is_dirty = false;

        Ok(camera)
    }
}
