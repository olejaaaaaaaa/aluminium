use std::sync::Arc;

use ash::vk::{self, Handle};

use crate::RasterPipeline;
use crate::core::Resolution;
use crate::resources::{Res, Resources};

/// The context of the currently running pass
pub struct PassContext {
    pub(crate) external_resources: Arc<Resources>,
    pub(crate) bindless: vk::DescriptorSet,
    pub(crate) scissor: vk::Rect2D,
    pub(crate) view: vk::Viewport,
    pub(crate) viewport: vk::Viewport,
    pub(crate) resolution: vk::Extent2D,
    pub(crate) pipeline: vk::Pipeline,
    pub(crate) layout: vk::PipelineLayout,
    pub(crate) device: ash::Device,
    pub(crate) cbuf: vk::CommandBuffer,
}

impl PassContext {
    pub unsafe fn set_viewport(&mut self, view: vk::Viewport) {
        self.view = self.view;
    }

    pub unsafe fn set_scissor(&mut self, scissor: vk::Rect2D) {
        self.scissor = scissor;
    }

    pub unsafe fn bind_pipeline(&self, handle: &Res<RasterPipeline>) {

        //let pipeline_cache = self.external_resources.pipeline_cache.read().unwrap();
        

        let scissors = vec![self.scissor];
        self.device.cmd_set_scissor(self.cbuf, 0, &scissors);

        let views = vec![self.view];
        self.device.cmd_set_viewport(self.cbuf, 0, &views);

        self.device
            .cmd_bind_pipeline(self.cbuf, vk::PipelineBindPoint::GRAPHICS, self.pipeline);
    }

    pub unsafe fn dispatch(&self, x: u32, y: u32, z: u32) {
        self.device.cmd_dispatch(self.cbuf, x, y, z);
    }

    // pub unsafe fn draw_mesh_instanced(&self, mesh: &Handle<Mesh>, transforms:
    // &[Handle<Transform>]) {

    // }

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
