use ash::vk;
use bytemuck::{Pod, Zeroable};

use crate::buffering::{PerFrameBuffer, PerFrameBufferBuilder};
use crate::core::{Device, VulkanResult};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct FrameData {
    resolution: [u32; 2],
    frame_index: u32,
    delta_time_sec: f32,
    time_sec: f32,
    _pad: [f32; 3],
}

pub struct FrameValues {
    global_time_sec: std::time::Instant,
    delta_time_sec: std::time::Instant,
    pub(crate) buffer: PerFrameBuffer,
    data: FrameData,
}

impl FrameValues {
    pub fn new(device: &Device, frame_count: usize) -> VulkanResult<Self> {
        let buffer = PerFrameBufferBuilder::new(device)
            .frame_count(frame_count)
            .buffer_size(size_of::<FrameData>() as u64)
            .usage(vk::BufferUsageFlags::UNIFORM_BUFFER)
            .build()?;

        Ok(Self {
            global_time_sec: std::time::Instant::now(),
            delta_time_sec: std::time::Instant::now(),
            data: FrameData {
                resolution: [0, 0],
                frame_index: 0,
                delta_time_sec: 0.0,
                time_sec: 0.0,
                _pad: [0.0, 0.0, 0.0],
            },
            buffer,
        })
    }

    pub fn set_resolution(&mut self, resolution: [u32; 2]) {
        self.data.resolution = resolution;
    }

    pub fn update(
        &mut self,
        device: &Device,
        image_index: u32,
        frame_index: u32,
    ) -> VulkanResult<()> {
        let buffer = self.buffer.get_mut(image_index);
        self.data.frame_index = frame_index;
        self.data.time_sec = self.global_time_sec.elapsed().as_secs_f32();
        self.data.delta_time_sec = self.delta_time_sec.elapsed().as_secs_f32();
        self.delta_time_sec = std::time::Instant::now();
        buffer.upload_data(device, &[self.data])?;
        Ok(())
    }
}
