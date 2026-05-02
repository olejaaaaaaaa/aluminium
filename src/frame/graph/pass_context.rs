use std::sync::Arc;

use ash::vk::{self};
use bytemuck::{Pod, Zeroable};
use crate::{RasterPipeline, Scissor, Texture, Viewport, resources::{IndexBuffer, Res, Resources, VertexBuffer}};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Debug)]
pub struct PushConstants {
    tex_idx: [u32; 8],
    user_data: [u8; 96],
}

#[derive(Default)]
pub struct RuntimeData {
    pub push: Option<PushConstants>,
    pub addition_sets: Vec<vk::DescriptorSet>,
    pub bind_point: Option<vk::PipelineBindPoint>,
    pub pipeline: Option<vk::Pipeline>,
    pub viewport: Option<vk::Viewport>,
    pub scissor: Option<vk::Rect2D>,
    pub layout: Option<vk::PipelineLayout>,
}

pub struct StaticData {
    //pub per_frame: vk::DescriptorSet,
    pub bindless: vk::DescriptorSet,
    pub resolution: vk::Extent2D,
    pub device: ash::Device,
    pub cbuf: vk::CommandBuffer
}

enum Pipeline<'a> {
    Raster(&'a Res<RasterPipeline>)
}

impl<'a> Into<Pipeline<'a>> for &'a Res<RasterPipeline> {
    fn into(self) -> Pipeline<'a> {
        Pipeline::Raster(self)
    }
}

/// The context of the currently running pass
pub struct PassContext {
    pub(crate) external_resources: Arc<Resources>,
    pub(crate) static_data: StaticData,
    pub(crate) runtime_data: RuntimeData
}

impl PassContext {

    pub unsafe fn bind_texture(&mut self, textures: &[&Res<Texture>]) {
        let push = self.runtime_data.push.as_mut().unwrap();
        for i in 0..textures.len() {
            push.tex_idx[i] = self.external_resources.textures.read().get(textures[i].key).unwrap().index;
        }
        
    }

    pub unsafe fn draw_fullscreen(&self) {

        let device = &self.static_data.device;
        let cbuf = self.static_data.cbuf;
        //let per_frame_set = self.static_data.per_frame;
        let bindless = self.static_data.bindless;
        let pipeline = self.runtime_data.pipeline.expect("Required Bind pipeline");
        let layout = self.runtime_data.layout.unwrap();
        let bind_point = self.runtime_data.bind_point.unwrap();

        let push = self.runtime_data.push.unwrap_or(PushConstants {
            tex_idx: [0u32; 8],
            user_data: [0u8; 96]
        });

        device.cmd_bind_pipeline(cbuf, bind_point, pipeline);

        if let Some(viewport) = self.runtime_data.viewport {
            let views = [viewport];
            device.cmd_set_viewport(cbuf, 0, &views);
        }

        if let Some(scissor) = self.runtime_data.scissor {
            let scissors = [scissor];
            device.cmd_set_scissor(cbuf, 0, &scissors);
        }

        device.cmd_push_constants(
            cbuf,
            layout,
            vk::ShaderStageFlags::FRAGMENT | vk::ShaderStageFlags::VERTEX,
            0,
            bytemuck::bytes_of(&push),
        );

        device.cmd_bind_descriptor_sets(
            cbuf,
            bind_point,
            layout,
            0,
            &[bindless],
            &[],
        );

        device.cmd_draw(cbuf, 3, 1, 0, 0);
    }

    pub unsafe fn set_viewport(&mut self, viewport: Viewport) {
        profiling::scope!("PassContext::set_viewport");

        let resolution = self.static_data.resolution;

        let viewport = match viewport {
            Viewport::FullRes => vk::Viewport::default()
                .height(resolution.height as f32)
                .width(resolution.width as f32)
                .min_depth(0.0)
                .max_depth(1.0)
                .x(0.0)
                .y(0.0),
            Viewport::HalfRes => vk::Viewport::default()
                .height(resolution.height as f32 / 2.0)
                .width(resolution.width as f32 / 2.0)
                .min_depth(0.0)
                .max_depth(1.0)
                .x(0.0)
                .y(0.0),
            Viewport::QuarterRes => vk::Viewport::default()
                .height(resolution.height as f32 / 4.0)
                .width(resolution.width as f32 / 4.0)
                .min_depth(0.0)
                .max_depth(1.0)
                .x(0.0)
                .y(0.0),
            Viewport::Custom(width, height) => vk::Viewport::default()
                .height(width as f32)
                .width(height as f32)
                .min_depth(0.0)
                .max_depth(1.0)
                .x(0.0)
                .y(0.0),
        };

        self.runtime_data.viewport = Some(viewport);
    }

    pub unsafe fn set_scissor(&mut self, scissor: Scissor) {
        profiling::scope!("PassContext::set_scissor");

        let resolution = self.static_data.resolution;

        let scissor = match scissor {
            Scissor::FullRes => vk::Rect2D::default()
                .extent(vk::Extent2D {
                    width: resolution.width,
                    height: resolution.height,
                })
                .offset(vk::Offset2D { x: 0, y: 0 }),
            Scissor::HalfRes => vk::Rect2D::default()
                .extent(vk::Extent2D {
                    width: resolution.width / 2,
                    height: resolution.height / 2,
                })
                .offset(vk::Offset2D { x: 0, y: 0 }),
            Scissor::QuarterRes => vk::Rect2D::default()
                .extent(vk::Extent2D {
                    width: resolution.width / 4,
                    height: resolution.height / 4,
                })
                .offset(vk::Offset2D { x: 0, y: 0 }),
            Scissor::Custom(width, height) => vk::Rect2D::default()
                .extent(vk::Extent2D { width, height })
                .offset(vk::Offset2D { x: 0, y: 0 }),
        };
        self.runtime_data.scissor = Some(scissor);
    }

    pub unsafe fn bind_pipeline<'a, P: Into<Pipeline<'a>>>(&mut self, handle: P) {
        profiling::scope!("PassContext::bind_pipeline");

        match handle.into() {
            Pipeline::Raster(handle) => {
                let cache = self.external_resources.pipeline_cache.read();
                let pipeline = cache.raster_pipelines.0.read();
                let pipeline = pipeline.get(handle.key).unwrap();

                let layout = cache.pipeline_layout.0.read().get(pipeline.layout.key).unwrap().raw.clone();

                self.runtime_data.layout = Some(layout);
                self.runtime_data.pipeline = Some(pipeline.pipeline.raw);
                self.runtime_data.bind_point = Some(vk::PipelineBindPoint::GRAPHICS);
            }
        }
        
    }

    pub unsafe fn push_constants<T: Pod + Zeroable>(&mut self, data: T) {
        let data = bytemuck::bytes_of(&data);
        let mut out = [0u8; 96];

        for (index, data) in data.iter().enumerate() {
            out[index] = *data;
        }

        let push = PushConstants {
            tex_idx: [0; 8],
            user_data: out,
        };

        self.runtime_data.push = Some(push);
    }

    pub unsafe fn draw_indexed(&self, vertices: &Res<VertexBuffer>, indices: &Res<IndexBuffer>) {

        let binding = self.external_resources.vertices.0.read();
        let vertex_buffer = binding.get(vertices.key).unwrap();

        let binding = self.external_resources.indices.0.read();
        let index_buffer = binding.get(indices.key).unwrap();

        let device = &self.static_data.device;
        let cbuf = self.static_data.cbuf;
        //let per_frame_set = self.static_data.per_frame;
        let bindless = self.static_data.bindless;
        let pipeline = self.runtime_data.pipeline.expect("Required Bind pipeline");
        let layout = self.runtime_data.layout.unwrap();
        let bind_point = self.runtime_data.bind_point.unwrap();

        let push = self.runtime_data.push.unwrap_or(PushConstants {
            tex_idx: [0u32; 8],
            user_data: [0u8; 96]
        });

        device.cmd_bind_pipeline(cbuf, bind_point, pipeline);

        if let Some(viewport) = self.runtime_data.viewport {
            let views = [viewport];
            device.cmd_set_viewport(cbuf, 0, &views);
        }

        if let Some(scissor) = self.runtime_data.scissor {
            let scissors = [scissor];
            device.cmd_set_scissor(cbuf, 0, &scissors);
        }

        device.cmd_push_constants(
            cbuf,
            layout,
            vk::ShaderStageFlags::FRAGMENT | vk::ShaderStageFlags::VERTEX,
            0,
            bytemuck::bytes_of(&push),
        );

        let mut sets = vec![bindless];
        sets.extend(self.runtime_data.addition_sets.clone());

        device.cmd_bind_descriptor_sets(
            cbuf,
            bind_point,
            layout,
            0,
            &sets,
            &[],
        );

        device
            .cmd_bind_vertex_buffers(cbuf, 0, &[vertex_buffer.buffer.raw], &[0]);

        device.cmd_bind_index_buffer(
            cbuf,
            index_buffer.buffer.raw,
            0,
            vk::IndexType::UINT32,
        );

        device
            .cmd_draw_indexed(cbuf, index_buffer.buffer.count, 1, 0, 0, 0);
    }
}
