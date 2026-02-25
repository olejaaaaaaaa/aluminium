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
use crate::resource_manager::{FrameBufferHandle, Renderable, ResourceManager};

pub type Execute = dyn Fn(&PassContext, &[Renderable]);

pub enum Pass {
    Rt(RtPass),
    Raster(RasterPass),
    Compute(ComputePass),
    Present(PresentPass),
}

impl Pass {
    pub fn pipeline_layout(&self, resources: &ResourceManager) -> vk::PipelineLayout {
        match self {
            Pass::Rt(_rt_pass) => todo!(),
            Pass::Raster(raster_pass) => {
                resources
                    .low_level
                    .read()
                    .unwrap()
                    .pipeline_layout
                    .get(raster_pass.layout)
                    .unwrap()
                    .raw
            },
            Pass::Compute(_compute_pass) => todo!(),
            Pass::Present(present_pass) => {
                resources
                    .low_level
                    .read()
                    .unwrap()
                    .pipeline_layout
                    .get(present_pass.layout)
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
                    .low_level
                    .read()
                    .unwrap()
                    .raster_pipeline
                    .get(raster_pass.pipeline)
                    .unwrap()
                    .raw
            },
            Pass::Compute(_compute_pass) => todo!(),
            Pass::Present(present_pass) => {
                resources
                    .low_level
                    .read()
                    .unwrap()
                    .raster_pipeline
                    .get(present_pass.pipeline)
                    .unwrap()
                    .raw
            },
        }
    }

    pub fn framebuffer(&self) -> FrameBufferHandle {
        match self {
            Pass::Rt(_rt_pass) => todo!(),
            Pass::Raster(_raster_pass) => todo!(),
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
