use ash::vk;

use crate::core::Resolution;
use crate::resource_manager::{Renderable, ResourceManager};

/// The context of the currently running pass
pub struct PassContext<'a> {
    pub(crate) resources: &'a ResourceManager,
    pub(crate) bindless_set: vk::DescriptorSet,
    pub(crate) resolution: vk::Extent2D,
    pub(crate) pipeline: vk::Pipeline,
    pub(crate) layout: vk::PipelineLayout,
    pub(crate) device: ash::Device,
    pub(crate) cbuf: vk::CommandBuffer,
}

impl<'a> PassContext<'a> {
    pub unsafe fn resolution(&self) -> [u32; 2] {
        self.resolution.into_array()
    }

    pub unsafe fn set_viewport(&self, view: Option<vk::Viewport>) {
        let view = view.unwrap_or(
            vk::Viewport::default()
                .height(self.resolution.height as f32)
                .width(self.resolution.width as f32)
                .x(0.0)
                .y(0.0),
        );
        let views = vec![view];
        self.device.cmd_set_viewport(self.cbuf, 0, &views);
    }

    pub unsafe fn set_scissor(&self, scissor: Option<vk::Rect2D>) {
        let scissor = scissor.unwrap_or(vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: vk::Extent2D::default()
                .width(self.resolution.width)
                .height(self.resolution.height),
        });

        let scissors = vec![scissor];
        self.device.cmd_set_scissor(self.cbuf, 0, &scissors);
    }

    pub unsafe fn bind_pipeline(&self) {
        self.device
            .cmd_bind_pipeline(self.cbuf, vk::PipelineBindPoint::GRAPHICS, self.pipeline);
    }

    pub unsafe fn bind_bindless(&self) {
        self.device.cmd_bind_descriptor_sets(
            self.cbuf,
            vk::PipelineBindPoint::GRAPHICS,
            self.layout,
            0,
            &[self.bindless_set],
            &[],
        );
    }

    pub unsafe fn bind_material(&self, _renderable: &Renderable) {}

    pub unsafe fn dispatch(&self) {}

    pub unsafe fn draw_mesh(&self, renderable: &Renderable) {
        let mesh = self.resources.get_mesh(renderable.mesh);

        self.device
            .cmd_bind_vertex_buffers(self.cbuf, 0, &[mesh.vertex_buffer.raw], &[0]);

        self.device
            .cmd_draw(self.cbuf, mesh.vertex_buffer.vertex_count, 1, 0, 0);
    }

    pub unsafe fn draw_fullscreen_triangle(&self) {
        self.device.cmd_draw(self.cbuf, 3, 1, 0, 0);
    }
}
