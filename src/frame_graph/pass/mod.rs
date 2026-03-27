mod present;

use ash::vk;
pub use present::*;

mod compute;
pub use compute::ComputePass;

mod raster;
pub use raster::*;

mod rt;
pub use rt::RtPass;

use super::PassContext;
use crate::resources::Resources;

pub struct DrawCallback(Box<dyn FnOnce(&PassContext) + Send + 'static>);
pub struct SyncCallback(Box<dyn FnOnce(vk::CommandBuffer) + Send + 'static>);

impl DrawCallback {
    pub unsafe fn new(callback: impl FnOnce(&PassContext) + Send + 'static) -> Self {
        Self(Box::new(callback))
    }

    pub(crate) fn empty() -> Self {
        Self(Box::new(|_| {}))
    }
}
pub enum Pass {
    // TODO:
    // Rt(RtPass),
    Raster(RasterPass),
    // TODO:
    // Compute(ComputePass),
    Present(PresentPass),
}
