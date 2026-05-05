use std::sync::Arc;

use ash::vk;

use crate::core::{GpuBuffer, GpuBufferBuilder};
use crate::render_context::RenderContext;
use crate::resources::{Create, Resources};
use crate::{Res, VulkanResult};

pub enum IndexSource<'a> {
    U32(&'a [u32]),
    U16(&'a [u16]),
}

impl<'a> From<&'a [u32]> for IndexSource<'a> {
    fn from(value: &'a [u32]) -> Self {
        IndexSource::U32(value)
    }
}

impl<'a> From<&'a [u16]> for IndexSource<'a> {
    fn from(value: &'a [u16]) -> Self {
        IndexSource::U16(value)
    }
}

impl<'a> From<&'a Vec<u32>> for IndexSource<'a> {
    fn from(value: &'a Vec<u32>) -> Self {
        IndexSource::U32(value.as_slice())
    }
}

impl<'a> From<&'a Vec<u16>> for IndexSource<'a> {
    fn from(value: &'a Vec<u16>) -> Self {
        IndexSource::U16(value.as_slice())
    }
}

pub struct IndexBuffer {
    pub(crate) buffer: GpuBuffer,
    pub(crate) count: u32,
    pub(crate) ty: vk::IndexType,
}

pub struct IndexBufferDesc<'a> {
    pub(crate) ty: vk::IndexType,
    pub(crate) count: u32,
    pub(crate) indices: &'a [u8],
}

impl<'a> IndexBufferDesc<'a> {
    pub fn new<I: Into<IndexSource<'a>>>(indices: I) -> Self {
        match indices.into() {
            IndexSource::U16(indices) => {
                return Self {
                    indices: bytemuck::cast_slice(indices),
                    count: indices.len() as u32,
                    ty: vk::IndexType::UINT16,
                };
            },
            IndexSource::U32(indices) => {
                return Self {
                    indices: bytemuck::cast_slice(indices),
                    count: indices.len() as u32,
                    ty: vk::IndexType::UINT32,
                };
            },
        };
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

        let key = resources.indices.insert(IndexBuffer {
            buffer: index_buffer,
            count: desc.count,
            ty: desc.ty,
        });

        Ok(resources.make_handle(ctx, key))
    }
}
