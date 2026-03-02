#[repr(C)]
#[derive(Clone, Copy)]
pub struct TriangleVertex {
    pos: [f32; 3],
}

#[repr(C)]
pub struct Bvh {}

impl Bvh {
    pub fn new() -> Self {
        Self {}
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct BVHNode {}
