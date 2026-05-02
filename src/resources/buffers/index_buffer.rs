use std::sync::Arc;

use ash::vk;

use crate::{Res, VulkanResult, core::{GpuBuffer, GpuBufferBuilder}, render_context::RenderContext, resources::{Create, Resources}};

pub enum IndexType {
    U32,
    U16
}

pub struct IndexBuffer {
    pub(crate) buffer: GpuBuffer,
    pub(crate) ty: IndexType
}

pub struct IndexBufferDesc<'a> {
    indices: &'a [u32],
}

impl<'a> IndexBufferDesc<'a> {
    pub fn new(indices: &'a [u32]) -> Self {
        Self { indices }
    }
}

impl Create for IndexBuffer {
    type Desc<'a> = IndexBufferDesc<'a>;
    fn create(
        ctx: &Arc<RenderContext>,
        resources: &Arc<Resources>,
        desc: Self::Desc<'_>,
    ) -> VulkanResult<Res<Self>> {

        let size = std::mem::size_of_val(desc.indices) as u64;

        let mut index_buffer = GpuBufferBuilder::cpu_only(&ctx.device)
            .size(size)
            .usage(vk::BufferUsageFlags::INDEX_BUFFER)
            .build()?;

        index_buffer.upload_data(desc.indices)?;

        let key = resources.indices.0.write().insert(IndexBuffer {
            buffer: index_buffer,
            ty: IndexType::U32
        });

        Ok(resources.make_handle(ctx, key))
    }
}