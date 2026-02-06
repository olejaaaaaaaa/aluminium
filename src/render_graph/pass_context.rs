use ash::vk;
use winit::window::Window;

use crate::core::{Device, VulkanResult};
use crate::render_graph::resources::RenderGraphResources;
use crate::resource_manager::Renderable;
use crate::Material;

/// The context of the currently running pass
pub struct PassContext {
    pub(crate) bindless_set: vk::DescriptorSet,
    pub(crate) resolution: vk::Extent2D,
    pub(crate) pipeline: vk::Pipeline,
    pub(crate) layout: vk::PipelineLayout,
    pub(crate) device: ash::Device,
    pub(crate) cbuf: vk::CommandBuffer,
}

impl PassContext {
    pub fn bind_pipeline(&self) -> VulkanResult<()> {
        unsafe { 
            let view = vk::Viewport::default()
                .height(self.resolution.height as f32)
                .width(self.resolution.width as f32)
                .x(0.0)
                .y(0.0);

            let scissor = vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: vk::Extent2D::default()
                    .width(self.resolution.width)
                    .height(self.resolution.height)
            };

            let views = vec![view];
            let scissors = vec![scissor];

            self.device.cmd_set_viewport(self.cbuf, 0, &views);
            self.device.cmd_set_scissor(self.cbuf, 0, &scissors);
            self.device.cmd_bind_pipeline(self.cbuf, vk::PipelineBindPoint::GRAPHICS, self.pipeline) 
        };
        Ok(())
    }

    pub fn bind_bindless(&self) -> VulkanResult<()> {
        Ok(())
    }

    pub fn bind_material(&self, renderable: &Renderable) -> VulkanResult<()> {
        Ok(())
    }

    pub fn dispatch(&self) -> VulkanResult<()> {
        Ok(())
    }

    pub fn draw_mesh(&self, renderable: &Renderable) -> VulkanResult<()> {
        Ok(())
    }

    pub fn draw_fullscreen_triangle(&self) -> VulkanResult<()> {
        unsafe {
            self.device.cmd_draw(self.cbuf, 3, 1, 0, 0);
        }
        Ok(())
    }
}
