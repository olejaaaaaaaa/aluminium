use std::sync::{Arc, Weak};

use ash::vk;
use bytemuck::{Pod, Zeroable};

use crate::core::{Device, VulkanResult};
use crate::per_frame::{PerFrameBuffer, PerFrameBufferBuilder};
use crate::resources::{Create, Destroy, LinearPool, Res, ResourceKey, Resources};

pub const MAX_TRANSFORMS: usize = 100;

/// Transform for Mesh
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct TransformDesc {
    pub mvp: [[f32; 4]; 4],
}

impl TransformDesc {
    /// identity matrix
    pub fn identity() -> Self {
        Self {
            mvp: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0f32],
            ]
        }
    }

    pub fn from(mvp: [[f32; 4]; 4]) -> Self {
        Self {
            mvp
        }
    }
}

/// Transform for Mesh
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Transform {
    pub mvp: [[f32; 4]; 4],
}

impl Transform {
    /// identity matrix
    pub fn identity() -> Self {
        Self {
            mvp: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0f32],
            ]
        }
    }
}

impl Destroy for Transform {
    fn destroy(_handle: ResourceKey, _ctx: Weak<crate::render_context::RenderContext>, _resources: Weak<Resources>) {}
}

impl Create for Transform {
    type Desc<'a> = TransformDesc;

    fn create(ctx: &Arc<crate::render_context::RenderContext>, resources: &Arc<Resources>, desc: Self::Desc<'_>) -> VulkanResult<Res<Self>> {
        let mut transforms = resources.transforms.try_write().expect("Err write lock");
        transforms.is_dirty = true;

        let handle = transforms.pool.insert(
            Arc::downgrade(ctx),
            Arc::downgrade(resources),
            Transform {
                mvp: desc.mvp,
            },
        );

        Ok(handle)
    }
}

pub struct TransformPool {
    pub is_dirty: bool,
    pub pool: LinearPool<Transform>,
    pub buffer: PerFrameBuffer,
}

impl TransformPool {
    pub fn new(device: &Device, frame_count: usize) -> VulkanResult<Self> {

        let mut buffer = PerFrameBufferBuilder::new(device)
            .buffer_size((size_of::<Transform>() * MAX_TRANSFORMS) as u64)
            .frame_count(frame_count)
            .usage(vk::BufferUsageFlags::STORAGE_BUFFER)
            .build()?;

        let data = vec![Transform::identity(); MAX_TRANSFORMS];

        for i in 0..frame_count {
            let buffer = buffer.get_mut(i as u32);
            buffer.upload_data(&data)?;
        }

        Ok(Self {
            buffer,
            pool: LinearPool::new(MAX_TRANSFORMS),
            is_dirty: false,
        })
    }

    pub fn update(&mut self, image_index: u32) -> VulkanResult<()> {
        let slice = self.pool.as_slice();
        for (i, t) in slice.iter().enumerate() {
            println!("transform[{}] translation: {:?}", i, [t.mvp[3][0], t.mvp[3][1], t.mvp[3][2]]);
        }
        let buffer = self.buffer.get_mut(0);
        buffer.upload_data(slice)?;
        Ok(())
    }

    pub fn destroy(&mut self, device: &Device) {
        self.buffer.destroy(device);
    }
}
