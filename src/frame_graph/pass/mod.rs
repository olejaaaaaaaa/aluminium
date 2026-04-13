mod present;

pub use present::*;

mod compute;
pub use compute::ComputePass;

mod raster;
pub use raster::*;

use super::PassContext;
use crate::frame_graph::RenderTarget;
use crate::Handle;

pub enum Pass<'frame> {
    Raster(RasterPass<'frame>),
    Compute(ComputePass<'frame>),
    Present(PresentPass<'frame>),
}

pub trait IntoPass<'a, T: Copy> {
    fn into_pass(self) -> (Pass<'a>, T);
}

