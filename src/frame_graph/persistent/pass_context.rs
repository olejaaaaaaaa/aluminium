use std::sync::Arc;

use ash::vk::{self};
use bytemuck::{Pod, Zeroable};

use crate::frame_graph::{Scissor, Viewport};
use crate::resources::{Res, Resources};
use crate::{FrameGraphUniform, Handle, Mesh, RasterPipeline, Transform, TransientTexture};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct PushConstants {
    transform_idx: u32,
    tex_idx: [u32; 8],
    user_data: [u8; 92],
}

/// The context of the currently running pass
pub struct PassContext {
    pub(crate) external_resources: Arc<Resources>,
    pub(crate) push: Option<PushConstants>,
    pub(crate) per_frame_set: vk::DescriptorSet,
    pub(crate) layout: Option<vk::PipelineLayout>,
    pub(crate) resolution: vk::Extent2D,
    pub(crate) device: ash::Device,
    pub(crate) cbuf: vk::CommandBuffer,
}

impl PassContext {
    pub unsafe fn set_viewport(&self, viewport: Viewport) {
        profiling::scope!("PassContext::set_viewport");
        let viewport = match viewport {
            Viewport::FullRes => vk::Viewport::default()
                .height(self.resolution.height as f32)
                .width(self.resolution.width as f32)
                .max_depth(0.0)
                .max_depth(1.0)
                .x(0.0)
                .y(0.0),
            Viewport::HalfRes => vk::Viewport::default()
                .height(self.resolution.height as f32 / 2.0)
                .width(self.resolution.width as f32 / 2.0)
                .max_depth(0.0)
                .max_depth(1.0)
                .x(0.0)
                .y(0.0),
            Viewport::QuarterRes => vk::Viewport::default()
                .height(self.resolution.height as f32 / 4.0)
                .width(self.resolution.width as f32 / 4.0)
                .max_depth(0.0)
                .max_depth(1.0)
                .x(0.0)
                .y(0.0),
            Viewport::Custom(width, height) => vk::Viewport::default()
                .height(width as f32)
                .width(height as f32)
                .max_depth(0.0)
                .max_depth(1.0)
                .x(0.0)
                .y(0.0),
        };
        let viewports = vec![viewport];
        self.device.cmd_set_viewport(self.cbuf, 0, &viewports);
    }

    pub unsafe fn set_scissor(&self, scissor: Scissor) {
        profiling::scope!("PassContext::set_scissor");
        let scissor = match scissor {
            Scissor::FullRes => vk::Rect2D::default()
                .extent(vk::Extent2D {
                    width: self.resolution.width,
                    height: self.resolution.height,
                })
                .offset(vk::Offset2D { x: 0, y: 0 }),
            Scissor::HalfRes => vk::Rect2D::default()
                .extent(vk::Extent2D {
                    width: self.resolution.width / 2,
                    height: self.resolution.height / 2,
                })
                .offset(vk::Offset2D { x: 0, y: 0 }),
            Scissor::QuarterRes => vk::Rect2D::default()
                .extent(vk::Extent2D {
                    width: self.resolution.width / 4,
                    height: self.resolution.height / 4,
                })
                .offset(vk::Offset2D { x: 0, y: 0 }),
            Scissor::Custom(width, height) => vk::Rect2D::default()
                .extent(vk::Extent2D { width, height })
                .offset(vk::Offset2D { x: 0, y: 0 }),
        };
        let scissors = vec![scissor];
        self.device.cmd_set_scissor(self.cbuf, 0, &scissors);
    }

    pub unsafe fn begin_rendering(&self, colors: &[Handle<TransientTexture>], depth: Option<Handle<TransientTexture>>) {

    }

    pub unsafe fn bind_set(&self, value: u32) {

    }

    pub unsafe fn bind_pipeline(&mut self, handle: &Res<RasterPipeline>) {
        profiling::scope!("PassContext::bind_pipeline");
        let cache = self.external_resources.pipeline_cache.read();
        let pipeline = cache.raster_pipelines.get(handle);
        let layout = cache.pipeline_layout.get(&pipeline.layout);
        self.device.cmd_bind_pipeline(
            self.cbuf,
            vk::PipelineBindPoint::GRAPHICS,
            pipeline.pipeline.raw,
        );
        self.layout = Some(layout.raw.clone());
    }

    pub unsafe fn dispatch(&self, x: u32, y: u32, z: u32) {
        profiling::scope!("PassContext::dispatch");
        self.device.cmd_dispatch(self.cbuf, x, y, z);
    }

    pub unsafe fn push_constants<T: Pod + Zeroable>(&mut self, data: T) {
        let data = bytemuck::bytes_of(&data);
        let mut out = [0u8; 92];

        for (index, data) in data.iter().enumerate() {
            out[index] = *data;
        }

        let push = PushConstants {
            transform_idx: 0,
            tex_idx: [0; 8],
            user_data: out,
        };

        self.push = Some(push);
    }

    pub unsafe fn draw_mesh(&self, mesh: &Res<Mesh>, transform: &Res<Transform>) {
        profiling::scope!("PassContext::draw_mesh");

        let binding = self.external_resources.meshes.read();
        let mesh = binding.get(mesh.key).unwrap();

        #[cfg(feature = "validation")]
        {
            assert!(self.layout.is_some(), "Pipeline must be bind before draw");
        }

        let layout = self.layout.unwrap();
        let mut push = self.push.unwrap();

        let index = self
            .external_resources
            .transforms
            .read()
            .pool
            .index(transform);

        push.transform_idx = index as u32;

        self.device.cmd_push_constants(
            self.cbuf,
            layout,
            vk::ShaderStageFlags::FRAGMENT | vk::ShaderStageFlags::VERTEX,
            0,
            bytemuck::bytes_of(&push),
        );

        self.device.cmd_bind_descriptor_sets(
            self.cbuf,
            vk::PipelineBindPoint::GRAPHICS,
            layout,
            0,
            &[self.per_frame_set],
            &[],
        );

        if let Some(index_buffer) = &mesh.index_buffer {
            self.device
                .cmd_bind_vertex_buffers(self.cbuf, 0, &[mesh.vertex_buffer.raw], &[0]);
            self.device.cmd_bind_index_buffer(
                self.cbuf,
                index_buffer.raw,
                0,
                vk::IndexType::UINT32,
            );
            self.device
                .cmd_draw_indexed(self.cbuf, index_buffer.count, 1, 0, 0, 0);
        } else {
            self.device
                .cmd_bind_vertex_buffers(self.cbuf, 0, &[mesh.vertex_buffer.raw], &[0]);
            self.device.cmd_draw(
                self.cbuf,
                mesh.vertex_buffer.count,
                mesh.instance_count,
                mesh.vertex_offset,
                mesh.instance_offset,
            );
        }
    }

    pub unsafe fn bind_uniforms(&self, uniform: Handle<FrameGraphUniform>) {

    }

    pub unsafe fn bind_texture(&self) {

    }

    pub unsafe fn draw(&self, vertex_count: u32) {
        self.device.cmd_draw(self.cbuf, vertex_count, 1, 0, 0);
    }
}
