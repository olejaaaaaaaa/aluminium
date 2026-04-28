mod raster;
pub use raster::*;

mod compiled;
pub use compiled::*;

use std::ops::Deref;
use std::sync::Arc;

use parking_lot::{Mutex, MutexGuard};

mod builder;
pub use builder::*;

mod compute;
pub use compute::*;

use crate::{Handle, TransientTexture};

use super::PassContext;

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

    pub fn texture_reads(&self) -> Vec<Handle<TransientTexture>> {
        match self {
            Pass::Raster(pass) => {
                pass.read_textures.iter().map(|x| x.0.clone()).collect()
            }
        }
    }

    pub fn texture_writes(&self) -> Vec<Handle<TransientTexture>> {
        match self {
            Pass::Raster(pass) => {
                pass.write_textures.clone()
            }
        }
    }
}