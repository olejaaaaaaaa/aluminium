use std::sync::{Arc};
use ash::vk;
use bytemuck::{Pod, Zeroable};
use parking_lot::{RwLockReadGuard, RwLockWriteGuard};

use crate::{Get, GetMut, Ref, RefMut, VulkanResult, core::{GpuBuffer, GpuBufferBuilder}, render_context::RenderContext, resources::{Create, Res, Resources}};


pub struct StorageBufferDesc<'a> {
    data: &'a [u8]
}

impl<'a> StorageBufferDesc<'a> {
    pub fn new<T: Pod + Zeroable>(data: &'a [T]) -> Self {
        Self { data: bytemuck::cast_slice(data) }
    }
}

impl Create for StorageBuffer {
    type Desc<'a> = StorageBufferDesc<'a>;
    fn create(
            ctx: &Arc<RenderContext>,
            resources: &Arc<Resources>,
            desc: Self::Desc<'_>,
        ) -> VulkanResult<Res<Self>> {

        let size = std::mem::size_of_val(desc.data) as u64;

        let mut storage_buffer = GpuBufferBuilder::cpu_only(&ctx.device)
            .size(size)
            .usage(vk::BufferUsageFlags::STORAGE_BUFFER)
            .build()?;

        storage_buffer.upload_data(desc.data)?;

        let key = resources.storage_buffers.write().insert(StorageBuffer {
            buffer: storage_buffer,
        });

        Ok(resources.make_handle(ctx, key))
    }
}

impl Get for Res<StorageBuffer> {
    type Output = StorageBuffer;
    fn try_get(&self) -> Option<Ref<'_, Self::Output>> {
        if let Some(guard) = self.resources.storage_buffers.try_read() {
            let mapped = RwLockReadGuard::map(guard, |x| {
                x.get(self.key).expect("Resource has already been destroyed")
            });
            return Some(Ref(mapped));
        } 
        None
    }
}


impl GetMut for Res<StorageBuffer> {
    type Output = StorageBuffer;
    fn try_get_mut(&self) -> Option<RefMut<'_, Self::Output>> {
        if let Some(guard) = self.resources.storage_buffers.try_write() {
            let mapped = RwLockWriteGuard::map(guard, |x| {
                x.get_mut(self.key).unwrap()
            });
            return Some(RefMut(mapped));
        }
        None
    }
}

pub struct StorageBuffer {
    pub(crate) buffer: GpuBuffer
}

impl StorageBuffer {
    pub fn upload_data<T: Pod>(&mut self, data: &[T]) -> VulkanResult<()> {
        self.buffer.upload_data(data)?;
        Ok(())
    }
}