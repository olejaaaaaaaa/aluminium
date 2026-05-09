mod raster;
pub use raster::*;

mod present;
pub use present::*;

mod builder;
pub use builder::*;

mod compute;
pub use compute::*;

use crate::{Handle, TransientTexture, PassContext};

pub enum Pass<'a> {
    Raster(RasterPass<'a>),
}

impl<'a> Pass<'a> {
    pub fn name(&self) -> String {
        match self {
            Pass::Raster(pass) => pass.name.clone(),
        }
    }

    pub fn texture_reads(&self) -> Vec<Handle<TransientTexture>> {
        match self {
            Pass::Raster(pass) => pass.read_textures.iter().map(|x| x.0.clone()).collect(),
        }
    }

    pub fn storage_texture_reads(&self) {

    }

    pub fn storage_texture_writes(&self) {
        
    }

    pub fn texture_writes(&self) -> Vec<Handle<TransientTexture>> {
        match self {
            Pass::Raster(pass) => pass.write_textures.clone(),
        }
    }
}
