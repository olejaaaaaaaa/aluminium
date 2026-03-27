use std::sync::{Arc, Weak};

use ash::vk;
use bytemuck::{Pod, Zeroable};

use crate::core::{Device, GpuBuffer, GpuBufferBuilder, VulkanResult};
use crate::per_frame::{PerFrameBuffer, PerFrameBufferBuilder};
use crate::resources::{Create, Destroy, LinearPool, Res, Resources};
use crate::VulkanError;

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
    fn destroy(handle: &Res<Self>, ctx: Weak<crate::render_context::RenderContext>, resources: Weak<Resources>) {
        
    }
}

impl Create for Transform {
    type Desc<'a> = TransformDesc;

    fn create(ctx: &Arc<crate::render_context::RenderContext>, resources: &Arc<Resources>, desc: Self::Desc<'_>) -> VulkanResult<Res<Self>> {
        let mut transforms = resources.transforms.write();
        transforms.is_dirty = true;
        
        let handle = transforms.pool.insert(Arc::downgrade(ctx), Arc::downgrade(resources), Transform { 
            rot: desc.rot, 
            scale: desc.scale, 
            pos: desc.pos, 
            _pad: [0.0, 0.0, 0.0, 0.0] 
        });

        Ok(handle)
    }
}

// impl Create for Transform {
//     type Desc<'a> = TransformDesc;
//     fn create(resources: &Resources, desc: Self::Desc<'_>) -> VulkanResult<Res<Self>> {
//         let mut transforms = resources.transforms.write();
//         let res = transforms.pool.insert(Transform {
//             rot: desc.rot,
//             scale: desc.scale,
//             pos: desc.pos,
//             _pad: [0.0, 0.0, 0.0, 0.0],
//         });

//         transforms.is_dirty = true;
//         Ok(res)
//     }
// }

pub struct TransformPool {
    pub is_dirty: bool,
    pub pool: LinearPool<Transform>,
    pub buffer: PerFrameBuffer,
}

impl TransformPool {
    pub fn new(device: &Device, frame_count: usize) -> VulkanResult<Self> {

        let mut buffer = PerFrameBufferBuilder::new(device)
            .buffer_size(size_of::<Transform>() as u64)
            .frame_count(frame_count)
            .usage(vk::BufferUsageFlags::UNIFORM_BUFFER)
            .build()?;

        let data = vec![Transform::identity(); MAX_TRANSFORMS];

        for i in 0..frame_count {
            let buffer = buffer.get_mut(i as u32);
            buffer.upload_data(device, &data)?;
        }

        Ok(Self {
            buffer,
            pool: LinearPool::new(MAX_TRANSFORMS),
            is_dirty: false,
        })
    }

    pub fn update(&mut self, device: &Device, image_index: u32) -> VulkanResult<()> {
        if self.is_dirty {
            let buffer = self.buffer.get_mut(image_index);
            buffer.upload_data(device, &self.pool.as_slice());
            self.is_dirty = false;
        }
        Ok(())
    }
}
