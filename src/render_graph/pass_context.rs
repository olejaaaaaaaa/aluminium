use ash::vk;

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
    pub fn set_viewport(&self, _view: Option<vk::Viewport>) {
        let view = vk::Viewport::default()
            .height(self.resolution.height as f32)
            .width(self.resolution.width as f32)
            .x(0.0)
            .y(0.0);
        let views = vec![view];
        unsafe {
            self.device.cmd_set_viewport(self.cbuf, 0, &views);
        }
    }

    pub fn set_scissor(&self, scissor: Option<vk::Rect2D>) {
        let scissor = vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: vk::Extent2D::default()
                .width(self.resolution.width)
                .height(self.resolution.height),
        };
        let scissors = vec![scissor];
        unsafe {
            self.device.cmd_set_scissor(self.cbuf, 0, &scissors);
        }
    }

    pub fn bind_pipeline(&self) {
        unsafe {
            self.device
                .cmd_bind_pipeline(self.cbuf, vk::PipelineBindPoint::GRAPHICS, self.pipeline);
        };
    }

    pub fn bind_bindless(&self) {
        unsafe {
            self.device.cmd_bind_descriptor_sets(
                self.cbuf,
                vk::PipelineBindPoint::GRAPHICS,
                self.layout,
                0,
                &[self.bindless_set],
                &[],
            );
        }
    }

    pub fn bind_material(&self, _renderable: &Renderable) {}

    pub fn dispatch(&self) {}

    pub fn draw_mesh(&self, renderable: &Renderable) {
        let mesh = self.resources.get_mesh(renderable.mesh);
        unsafe {
            self.device
                .cmd_bind_vertex_buffers(self.cbuf, 0, &[mesh.vertex_buffer.raw], &[0]);

            self.device
                .cmd_draw(self.cbuf, mesh.vertex_buffer.vertex_count, 1, 0, 0);
        }
    }

    pub fn draw_fullscreen_triangle(&self) {
        unsafe {
            self.device.cmd_draw(self.cbuf, 3, 1, 0, 0);
        }
    }
}
