mod present;

pub use present::*;

mod compute;
pub use compute::ComputePass;

mod raster;
pub use raster::*;

use crate::{Handle, frame_graph::{RenderTarget, RenderTargetsDesc}};

use super::PassContext;

type Execute<'a, T> = dyn FnOnce(&mut PassContext, &mut T) + Send + 'a;
type Setup<'a, T> = dyn FnOnce(&mut PassBuilder, &mut T) + Send + 'a;

pub enum Pass<'frame> {
    Raster(RasterPass<'frame>),
    Compute(ComputePass<'frame>),
    Present(PresentPass<'frame>),
}

pub struct PassBuilder<'a> {
    pub(crate) reads: Vec<bool>,
    pub(crate) writes: Vec<bool>,
    pub(crate) render_target_desc: Option<RenderTargetsDesc<'a>>
}

impl<'a> PassBuilder<'a> {
    pub fn read(&mut self, handle: bool) -> Handle<bool> {
        //vec![]
        todo!()
    }

    pub fn write(&mut self, handle: bool) -> Handle<bool> {
        //vec![]
        todo!()
    }

    pub fn render_targets(&mut self, desc: RenderTargetsDesc<'a>) {
        
    }
}


