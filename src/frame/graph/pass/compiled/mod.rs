
mod builder;
pub use builder::*;

mod raster;
pub use raster::*;

pub enum CompiledPass<'frame> {
    Raster(CompiledRasterPass<'frame>)
}