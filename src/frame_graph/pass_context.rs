use std::sync::Arc;

use ash::vk::{self};
use bytemuck::{Pod, Zeroable};

use crate::frame_graph::{Scissor, Viewport};
use crate::resources::{Res, Resources, Texture, TextureView};
use crate::{Mesh, RasterPipeline};

/// The context of the currently running pass
pub struct PassContext {
    pub(crate) external_resources: Arc<Resources>,
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
                .x(0.0)
                .y(0.0),
            Viewport::HalfRes => vk::Viewport::default()
                .height(self.resolution.height as f32 / 2.0)
                .width(self.resolution.width as f32 / 2.0)
                .x(0.0)
                .y(0.0),
            Viewport::QuarterRes => vk::Viewport::default()
                .height(self.resolution.height as f32 / 4.0)
                .width(self.resolution.width as f32 / 4.0)
                .x(0.0)
                .y(0.0),
            Viewport::Custom(width, height) => vk::Viewport::default()
                .height(width as f32)
                .width(height as f32)
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

    pub unsafe fn bind_pipeline(&mut self, handle: &Res<RasterPipeline>) {
        profiling::scope!("PassContext::bind_pipeline");
        let cache = self.external_resources.pipeline_cache.read();
        let pipeline = cache.raster_pipelines.get(handle);
        let layout = cache.pipeline_layout.get(&pipeline.layout);
        self.device.cmd_bind_pipeline(self.cbuf, vk::PipelineBindPoint::GRAPHICS, pipeline.pipeline.raw);
        self.layout = Some(layout.raw.clone());
    }

    pub unsafe fn dispatch(&self, x: u32, y: u32, z: u32) {
        profiling::scope!("PassContext::dispatch");
        self.device.cmd_dispatch(self.cbuf, x, y, z);
    }

    pub unsafe fn push_constants<T: Pod + Zeroable>(&self, data: T) {

        assert!(size_of_val(&data) <= 64, "The maximum size of Push Constants is 64 bytes");
        assert!(size_of_val(&data) > 0, "Push Constants cannot be empty");

        #[cfg(feature = "validation")]
        {
            assert!(self.layout.is_some(), "Pipeline must be bind before draw");
        }

        let layout = self.layout.unwrap();

        #[repr(C)]
        #[derive(Clone, Copy, Pod, Zeroable)]
        struct PushConstants {
            transform_idx: u32,
            tex_idx: [u32; 8],
            user_data: [u8; 92]
        }
        
        let data = bytemuck::bytes_of(&data);
        let mut out = [0u8; 92];

        for (index, data) in data.iter().enumerate() {
            out[index] = *data;
        }

        let push = PushConstants {
            transform_idx: 0,
            tex_idx: [0; 8],
            user_data: out
        };

        self.device.cmd_push_constants(
            self.cbuf, 
            layout, 
            vk::ShaderStageFlags::FRAGMENT | vk::ShaderStageFlags::VERTEX, 
            0, 
            bytemuck::bytes_of(&push)
        );
    }

    // pub unsafe fn bind_texture(&self, slot: usize, texture: &Res<TextureView>) {

    // }

    pub unsafe fn draw_mesh(&self, mesh: &Res<Mesh>) {
        profiling::scope!("PassContext::draw_mesh");

        let binding = self.external_resources.meshes.read();
        let mesh = binding.get(mesh.key).unwrap();

        if let Some(index_buffer) = &mesh.index_buffer {
            self.device
                .cmd_bind_vertex_buffers(self.cbuf, 0, &[mesh.vertex_buffer.raw], &[0]);
            self.device
                .cmd_bind_index_buffer(self.cbuf, index_buffer.raw, 0, vk::IndexType::UINT32);
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

    pub unsafe fn draw(&self, vertex_count: u32) {
        self.device.cmd_draw(self.cbuf, vertex_count, 1, 0, 0);
    }
}
