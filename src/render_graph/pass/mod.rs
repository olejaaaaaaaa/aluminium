mod present;

use ash::vk;
pub use present::*;

mod desc;
pub use desc::PassDesc;

mod source;
pub use source::Source;

mod ops;
pub use ops::{LoadOp, StoreOp};

mod compute;
pub use compute::ComputePass;

mod raster;
pub use raster::*;

mod rt;
pub use rt::RtPass;

use super::PassContext;
use crate::render_graph::RenderGraphResource;
use crate::resource_manager::{FrameBufferHandle, Renderable, ResourceManager};

pub type Execute = dyn Fn(&PassContext, &[Renderable]);

pub enum Pass {
    Rt(RtPass),
    Raster(RasterPass),
    Compute(ComputePass),
    Present(PresentPass),
}

impl Pass {
    pub fn pipeline_layout(&self, resources: &mut ResourceManager) -> vk::PipelineLayout {
        match self {
            Pass::Rt(_rt_pass) => todo!(),
            Pass::Raster(raster_pass) => {
                resources
                    .get_layout(raster_pass.pipeline.as_ref().unwrap().pipeline_layout)
                    .unwrap()
                    .raw
            },
            Pass::Compute(_compute_pass) => todo!(),
            Pass::Present(present_pass) => {
                resources
                    .get_layout(present_pass.pipeline_layout)
                    .unwrap()
                    .raw
            },
        }
    }

    pub fn pipeline(&self, resources: &ResourceManager) -> vk::Pipeline {
        match self {
            Pass::Rt(_rt_pass) => todo!(),
            Pass::Raster(raster_pass) => {
                resources
                    .get_raster_pipeline(raster_pass.pipeline.as_ref().unwrap().pipeline)
                    .unwrap()
                    .raw
            },
            Pass::Compute(_compute_pass) => todo!(),
            Pass::Present(present_pass) => {
                resources
                    .get_raster_pipeline(present_pass.pipeline)
                    .unwrap()
                    .raw
            },
        }
    }

    pub fn framebuffer(&self) -> FrameBufferHandle {
        match self {
            Pass::Rt(_rt_pass) => todo!(),
            Pass::Raster(raster_pass) => raster_pass.pipeline.as_ref().unwrap().frame_buffer,
            Pass::Compute(_compute_pass) => todo!(),
            Pass::Present(_present_pass) => todo!(),
        }
    }

    pub fn is_present(&self) -> bool {
        match self {
            Pass::Rt(_rt_pass) => false,
            Pass::Raster(_raster_pass) => false,
            Pass::Compute(_compute_pass) => false,
            Pass::Present(_present_pass) => true,
        }
    }

    pub fn execute(&self) -> &Box<dyn Fn(&PassContext, &[Renderable]) + 'static> {
        match self {
            Pass::Raster(raster) => &raster.execute_fn,
            Pass::Rt(_rt_pass) => todo!(),
            Pass::Compute(_compute_pass) => todo!(),
            Pass::Present(present_pass) => &present_pass.execute_fn,
        }
    }

    pub fn writes(&self) -> Vec<RenderGraphResource> {
        match self {
            Pass::Raster(raster) => raster.writes.clone(),
            Pass::Rt(_rt) => Vec::new(),
            Pass::Compute(_comp) => Vec::new(),
            Pass::Present(_) => Vec::new(),
        }
    }

    pub fn reads(&self) -> &Vec<RenderGraphResource> {
        match self {
            Pass::Raster(raster) => &raster.reads,
            Pass::Rt(_rt_pass) => todo!(),
            Pass::Compute(_compute_pass) => todo!(),
            Pass::Present(present_pass) => &present_pass.reads,
        }
    }
}

impl From<RasterPass> for Pass {
    fn from(val: RasterPass) -> Self {
        Pass::Raster(val)
    }
}

impl From<ComputePass> for Pass {
    fn from(val: ComputePass) -> Self {
        Pass::Compute(val)
    }
}

impl From<RtPass> for Pass {
    fn from(val: RtPass) -> Self {
        Pass::Rt(val)
    }
}

impl From<PresentPass> for Pass {
    fn from(val: PresentPass) -> Self {
        Pass::Present(val)
    }
}
