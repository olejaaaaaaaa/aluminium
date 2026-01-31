use crate::core::GpuBuffer;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct AABBData {
    max: [f32; 3],
    mix: [f32; 3],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct AABBList {
    count: u32,
    indices: [u32; 1024],
}

pub struct AABB {
    is_dirty: bool,
    aabb_data: Vec<AABBData>,
    aabb_buffer: bool,
}

impl AABB {
    pub fn new() -> Self {
        Self {
            is_dirty: false,
            aabb_data: vec![],
            aabb_buffer: true,
        }
    }
}
