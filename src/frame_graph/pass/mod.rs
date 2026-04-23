mod raster;
use std::ops::Deref;
use std::sync::Arc;

use parking_lot::{Mutex, MutexGuard};
pub use raster::*;

mod compute;
pub use compute::*;

use super::PassContext;
use crate::frame_graph::RenderTarget;
use crate::Handle;

pub enum Pass<'a> {
    Raster(RasterPass<'a>),
}

impl<'a> Pass<'a> {
    pub fn name(&self) -> String {
        match self {
            Pass::Raster(pass) => {
                pass.name.clone()
            }
        }
    }
}