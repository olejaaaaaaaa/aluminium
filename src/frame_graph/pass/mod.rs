mod present;

pub use present::*;

mod compute;
pub use compute::ComputePass;

mod raster;
pub use raster::*;

mod rt;

use super::PassContext;

pub enum Pass {
    // TODO:
    // Rt(RtPass),
    Raster(RasterPass),
    // TODO:
    // Compute(ComputePass),
    Present(PresentPass),
}
