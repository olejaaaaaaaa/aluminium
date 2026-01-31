use ash::vk;
use winit::window::Window;

use crate::core::{Device, VulkanResult};
use crate::render_graph::resources::RenderGraphResources;
use crate::resource_manager::Renderable;
use crate::Material;

/// The context of the currently running pass
pub struct PassContext<'a> {
    pub(crate) raw_bindless_set: vk::DescriptorSet,
    pub(crate) resolution: vk::Extent2D,
    pub(crate) device: &'a Device,
    pub(crate) cbuf: vk::CommandBuffer,
}

impl<'a> PassContext<'a> {
    pub fn bind_pipeline(&self) -> VulkanResult<()> {
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
