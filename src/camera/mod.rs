use ash::vk;
use bytemuck::{Pod, Zeroable};

use crate::core::{Device, VulkanResult};
use crate::per_frame::{PerFrameBuffer, PerFrameBufferBuilder};

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
    is_dirty: bool,
    buffer: PerFrameBuffer,
    data: CameraData,
}

impl Camera {
    pub fn proj(&mut self) -> &[[f32; 4]; 4] {
        &self.data.proj
    }

    pub fn view(&mut self) -> &[[f32; 4]; 4] {
        &self.data.view
    }

    pub fn view_mut(&mut self) -> &mut [[f32; 4]; 4] {
        self.is_dirty = true;
        &mut self.data.view
    }

    pub fn proj_mut(&mut self) -> &mut [[f32; 4]; 4] {
        self.is_dirty = true;
        &mut self.data.proj
    }

    pub fn new(device: &Device, frame_count: usize) -> VulkanResult<Self> {
        let size = size_of::<CameraData>() as u64;

        let mut buffer = PerFrameBufferBuilder::new(device)
            .buffer_size(size)
            .frame_count(frame_count)
            .usage(vk::BufferUsageFlags::UNIFORM_BUFFER)
            .build()?;

        let data = CameraData::identity();

        for i in 0..frame_count {
            let buffer = buffer.get_mut(i as u32);
            buffer.upload_data(&[data])?;
        }

        Ok(Self {
            is_dirty: false,
            buffer,
            data,
        })
    }

    pub fn destroy(&mut self, device: &Device) {
        self.buffer.destroy(device);
    }
}
