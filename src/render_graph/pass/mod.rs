mod present;
use std::path::PathBuf;

pub use present::PresentPass;

mod compute;
pub use compute::ComputePass;

mod raster;
pub use raster::RasterPass;

mod rt;
pub use rt::RtPass;

use super::PassContext;
use crate::core::VulkanResult;
use crate::render_graph::RenderGraphResource;
use crate::resource_manager::Renderable;

pub type Execute = dyn Fn(&PassContext, &[Renderable]) -> VulkanResult<()>;

pub trait PushConstant {
    fn bytes(&self) -> &[u8];
}

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
            Pass::Rt(_) => todo!(),
            Pass::Compute(_) => todo!(),
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
