mod raster;
use std::ops::Deref;
use std::sync::Arc;

use parking_lot::{Mutex, MutexGuard};
pub use raster::*;

mod compute;
pub use compute::*;

use super::PassContext;
use crate::frame_graph::temporal::TemporalFrameGraphResources;
use crate::frame_graph::RenderTarget;
use crate::Handle;

pub enum Pass<'a> {
    Raster(RasterPass<'a>),
}

pub struct PassData<T: Clone + Default>(pub(crate) Arc<Mutex<T>>);

impl<T: Copy + Default> PassData<T> {
    pub fn get(&self) -> T {
        *self.0.lock()
    }
}