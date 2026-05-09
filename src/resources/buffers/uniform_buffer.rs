use crate::core::GpuBuffer;

pub struct UniformBuffer {
    buffers: Vec<GpuBuffer>,
    dirty_frames: usize
}
