use std::sync::{Arc, Weak};

use ash::vk;
use bytemuck::{Pod, Zeroable};

use crate::core::{Device, GpuBuffer, GpuBufferBuilder, VulkanResult};
use crate::resources::{Create, Destroy, LinearPool, Res, Resources};
use crate::ring_buffer::RingBuffer;
use crate::{VulkanError};

pub const MAX_TRANSFORMS: usize = 1_000;

/// Transform for Mesh
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct TransformDesc {
    /// Rot
    pub rot: [f32; 4],
    /// Scale
    pub scale: [f32; 4],
    /// Pos
    pub pos: [f32; 4],
}

impl TransformDesc {
    /// identity matrix
    pub fn identity() -> Self {
        Self {
            scale: [1.0, 1.0, 1.0, 0.0],
            rot: [0.0, 0.0, 0.0, 0.0],
            pos: [0.0, 0.0, 0.0, 0.0],
        }
    }
}

/// Transform for Mesh
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Transform {
    /// Rot
    pub rot: [f32; 4],
    /// Scale
    pub scale: [f32; 4],
    /// Pos
    pub pos: [f32; 4],
    /// Pad to 64 bytes
    pub _pad: [f32; 4],
}

impl Transform {
    /// identity matrix
    pub fn identity() -> Self {
        Self {
            scale: [1.0, 1.0, 1.0, 0.0],
            rot: [0.0, 0.0, 0.0, 0.0],
            pos: [0.0, 0.0, 0.0, 0.0],
            _pad: [0.0, 0.0, 0.0, 0.0],
        }
    }
}

impl Destroy for Transform {
    fn destroy(key: super::ResourceKey, resources: &Resources) {
        // nothing
    }
}

impl Create for Transform {
    type Desc<'a> = TransformDesc;
    fn create(resources: &Resources, desc: Self::Desc<'_>) -> VulkanResult<Res<Self>> {
        let binding = &resources.transforms;
        let mut transforms = binding.write().unwrap();
        let res = transforms.pool.insert(Transform {
            rot: desc.rot,
            scale: desc.scale,
            pos: desc.pos,
            _pad: [0.0, 0.0, 0.0, 0.0],
        });

        transforms.is_dirty = true;
        Ok(res)
    }
}

pub struct TransformPool {
    pub is_dirty: bool,
    pub pool: LinearPool<Transform>,
    pub buffer: RingBuffer,
}

impl TransformPool {
    pub fn new(device: &Device, frame_count: usize, root: Weak<Resources>) -> VulkanResult<Self> {
        let buffer = RingBuffer::new(
            device,
            (MAX_TRANSFORMS * size_of::<Transform>()) as u64,
            frame_count,
            vk::BufferUsageFlags::STORAGE_BUFFER,
        )?;

        let data = vec![Transform::identity(); MAX_TRANSFORMS * size_of::<Transform>() * frame_count];

        buffer.write(device, &data)?;

        Ok(Self {
            buffer,
            pool: LinearPool::new(root, MAX_TRANSFORMS),
            is_dirty: false,
        })
    }

    pub fn begin_frame(&mut self) {
        self.buffer.advance();
    }

    pub fn offset(&self) -> u64 {
        self.buffer.current_offset()
    }

    pub fn update(&mut self, device: &Device) -> VulkanResult<()> {
        if self.is_dirty {
            self.buffer.write(device, self.pool.as_slice())?;
            self.is_dirty = false;
        }
        Ok(())
    }
}
