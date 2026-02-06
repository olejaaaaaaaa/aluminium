mod present;
use std::path::PathBuf;

use ash::vk;
pub use present::PresentPass;

mod compute;
pub use compute::ComputePass;

mod raster;
pub use raster::RasterPass;

mod rt;
pub use rt::RtPass;

use super::PassContext;
use crate::core::{VulkanError, VulkanResult};
use crate::render_graph::RenderGraphResource;
use crate::resource_manager::{FrameBufferHandle, Renderable, ResourceManager};

pub type Execute = dyn Fn(&PassContext, &[Renderable]) -> VulkanResult<()>;

#[derive(Debug, Clone, Copy)]
pub enum LoadOp {
    Clear,
    Store,
    DontCare,
}

#[derive(Debug, Clone, Copy)]
pub enum StoreOp {
    Clear,
    Store,
    DontCare,
}

pub enum Pass {
    Rt(RtPass),
    Raster(RasterPass),
    Compute(ComputePass),
    Present(PresentPass),
}

impl Pass {

    pub fn pipeline_layout(&self, resources: &mut ResourceManager) -> vk::PipelineLayout {
        match self {
            Pass::Rt(rt_pass) => todo!(),
            Pass::Raster(raster_pass) => resources.get_layout(raster_pass.pipeline.as_ref().unwrap().pipeline_layout).unwrap().raw,
            Pass::Compute(compute_pass) => todo!(),
            Pass::Present(present_pass) => resources.get_layout(present_pass.pipeline.as_ref().unwrap().pipeline_layout).unwrap().raw,
        }
    }

    pub fn pipeline(&self, resources: &ResourceManager) -> vk::Pipeline {
        match self {
            Pass::Rt(rt_pass) => todo!(),
            Pass::Raster(raster_pass) => resources.get_raster_pipeline(raster_pass.pipeline.as_ref().unwrap().pipeline).unwrap().raw,
            Pass::Compute(compute_pass) => todo!(),
            Pass::Present(present_pass) => resources.get_raster_pipeline(present_pass.pipeline.as_ref().unwrap().pipeline).unwrap().raw,
        }
    }

    pub fn framebuffer(&self) -> FrameBufferHandle {
        match self {
            Pass::Rt(rt_pass) => todo!(),
            Pass::Raster(raster_pass) => raster_pass.pipeline.as_ref().unwrap().frame_buffer,
            Pass::Compute(compute_pass) => todo!(),
            Pass::Present(present_pass) => todo!(),
        }
    }

    pub fn is_present_pass(&self) -> bool {
        match self {
            Pass::Rt(rt_pass) => false,
            Pass::Raster(raster_pass) => false,
            Pass::Compute(compute_pass) => false,
            Pass::Present(present_pass) => true,
        }
    }

    pub fn execute(
        &self,
    ) -> &Box<dyn Fn(&PassContext, &[Renderable]) -> Result<(), VulkanError> + 'static> {
        match self {
            Pass::Raster(raster) => &raster.execute,
            Pass::Rt(rt_pass) => todo!(),
            Pass::Compute(compute_pass) => todo!(),
            Pass::Present(present_pass) => &present_pass.execute,
        }
    }

    pub fn shaders(&self) -> Vec<PathBuf> {
        match self {
            Pass::Raster(raster_pass) => {
                let pipeline = raster_pass.pipeline.as_ref().unwrap();
                vec![
                    pipeline.vertex_shader.clone(),
                    pipeline.fragment_shader.clone(),
                ]
            },
            Pass::Rt(rt_pass) => todo!(),
            Pass::Compute(compute_pass) => todo!(),
            Pass::Present(present_pass) => {
                let pipeline = present_pass.pipeline.as_ref().unwrap();
                vec![
                    pipeline.vertex_shader.clone(),
                    pipeline.fragment_shader.clone(),
                ]
            },
        }
    }

    pub fn writes(&self) -> Vec<RenderGraphResource> {
        match self {
            Pass::Raster(raster) => raster.writes.clone(),
            Pass::Rt(rt) => Vec::new(),
            Pass::Compute(comp) => Vec::new(),
            Pass::Present(_) => Vec::new(),
        }
    }

    pub fn reads(&self) -> &Vec<RenderGraphResource> {
        match self {
            Pass::Raster(raster) => &raster.reads,
            Pass::Rt(rt_pass) => todo!(),
            Pass::Compute(compute_pass) => todo!(),
            Pass::Present(present_pass) => &present_pass.reads,
        }
    }
}

impl Into<Pass> for RasterPass {
    fn into(self) -> Pass {
        Pass::Raster(self)
    }
}

impl Into<Pass> for ComputePass {
    fn into(self) -> Pass {
        Pass::Compute(self)
    }
}

impl Into<Pass> for RtPass {
    fn into(self) -> Pass {
        Pass::Rt(self)
    }
}

impl Into<Pass> for PresentPass {
    fn into(self) -> Pass {
        Pass::Present(self)
    }
}
