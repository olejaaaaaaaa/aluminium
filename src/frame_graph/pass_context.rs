use std::sync::Arc;

use ash::vk::{self, Handle};

use crate::core::Resolution;
use crate::frame_graph::{Scissor, Viewport};
use crate::resources::{Res, Resources};
use crate::RasterPipeline;

/// The context of the currently running pass
pub struct PassContext {
    pub(crate) external_resources: Arc<Resources>,
    pub(crate) bindless: vk::DescriptorSet,
    pub(crate) resolution: vk::Extent2D,
    pub(crate) pipeline: vk::Pipeline,
    pub(crate) layout: vk::PipelineLayout,
    pub(crate) device: ash::Device,
    pub(crate) cbuf: vk::CommandBuffer,
}

impl PassContext {
    pub unsafe fn set_viewport(&self, viewport: Viewport) {
        let viewport = match viewport {
            Viewport::FullRes => {
                vk::Viewport::default()
                    .height(self.resolution.height as f32)
                    .width(self.resolution.width as f32)
                    .x(0.0)
                    .y(0.0)
            },
            Viewport::HalfRes => {
                vk::Viewport::default()
                    .height(self.resolution.height as f32 / 2.0)
                    .width(self.resolution.width as f32 / 2.0)
                    .x(0.0)
                    .y(0.0)
            },
            Viewport::QuarterRes => {
                vk::Viewport::default()
                    .height(self.resolution.height as f32 / 4.0)
                    .width(self.resolution.width as f32 / 4.0)
                    .x(0.0)
                    .y(0.0)
            },
            Viewport::Custom(width, height) => {
                vk::Viewport::default()
                    .height(width as f32)
                    .width(height as f32)
                    .x(0.0)
                    .y(0.0)
            }
        };
        let viewports = vec![viewport];
        self.device.cmd_set_viewport(self.cbuf, 0, &viewports);
    }

    pub unsafe fn set_scissor(&self, scissor: Scissor) {
        let scissor = match scissor {
            Scissor::FullRes => {
                vk::Rect2D::default()
                    .extent(vk::Extent2D {
                        width: self.resolution.width,
                        height: self.resolution.height
                    })
                    .offset(vk::Offset2D {
                        x: 0,
                        y: 0
                    })
            },
            Scissor::HalfRes => {
                vk::Rect2D::default()
                    .extent(vk::Extent2D {
                        width: self.resolution.width / 2,
                        height: self.resolution.height / 2
                    })
                    .offset(vk::Offset2D {
                        x: 0,
                        y: 0
                    })
            },
            Scissor::QuarterRes => {
                vk::Rect2D::default()
                    .extent(vk::Extent2D {
                        width: self.resolution.width / 4,
                        height: self.resolution.height / 4
                    })
                    .offset(vk::Offset2D {
                        x: 0,
                        y: 0
                    })
            },
            Scissor::Custom(width, height) => {
                vk::Rect2D::default()
                    .extent(vk::Extent2D {
                        width: width,
                        height: height
                    })
                    .offset(vk::Offset2D {
                        x: 0,
                        y: 0
                    })
            }
        };
        let scissors = vec![scissor];
        self.device.cmd_set_scissor(self.cbuf, 0, &scissors);
    }

    // pub unsafe fn bind_pipeline(&self, handle: &Res<RasterPipeline>) {
    //     self.device
    //         .cmd_bind_pipeline(self.cbuf, vk::PipelineBindPoint::GRAPHICS, self.pipeline);
    // }

    pub unsafe fn dispatch(&self, x: u32, y: u32, z: u32) {
        self.device.cmd_dispatch(self.cbuf, x, y, z);
    }

    pub unsafe fn draw_mesh(
        &self,
        // mesh: &Handle<Mesh>,
        // transform: &Handle<Transform>,
    ) {
        // let resources = self.resources.assets.read().unwrap();
        // let mesh =
        // resources.mesh.get_mesh(renderable.mesh.clone());

        // self.device
        //     .cmd_bind_vertex_buffers(self.cbuf, 0,
        // &[mesh.vertex_buffer.raw], &[0]);

        // self.device
        //     .cmd_draw(self.cbuf, mesh.vertex_buffer.vertex_count,
        // 1, 0, 0);
    }

    pub unsafe fn draw(&self, vertex_count: u32) {
        self.device.cmd_draw(self.cbuf, vertex_count, 1, 0, 0);
    }
}
