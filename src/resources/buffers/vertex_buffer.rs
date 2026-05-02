use std::sync::Arc;

use ash::vk;
use bytemuck::{Pod, Zeroable};

use crate::{Res, VulkanResult, core::{GpuBuffer, GpuBufferBuilder}, render_context::RenderContext, resources::{Create, Resources}};

pub struct VertexBufferDesc<'a> {
    vertices: &'a [u8],
}

impl<'a> VertexBufferDesc<'a> {
    pub fn new<T: Pod + Zeroable>(vertices: &'a [T]) -> Self {
        Self { vertices: bytemuck::cast_slice(vertices) }
    }
}

pub struct VertexBuffer {
    pub(crate) buffer: GpuBuffer
}

impl Create for VertexBuffer {
    type Desc<'a> = VertexBufferDesc<'a>;
    fn create(
        ctx: &Arc<RenderContext>,
        resources: &Arc<Resources>,
        desc: Self::Desc<'_>,
    ) -> VulkanResult<Res<Self>> {

        let size = std::mem::size_of_val(desc.vertices) as u64;

        let mut vertex_buffer = GpuBufferBuilder::cpu_only(&ctx.device)
            .size(size)
            .usage(vk::BufferUsageFlags::VERTEX_BUFFER)
            .build()?;

        vertex_buffer.upload_data(desc.vertices)?;

        let key = resources.vertices.0.write().insert(VertexBuffer {
            buffer: vertex_buffer,
        });

        Ok(resources.make_handle(ctx, key))
    }
}
