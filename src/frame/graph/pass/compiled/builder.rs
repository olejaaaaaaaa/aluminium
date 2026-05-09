use super::CompiledRasterPassBuilder;

pub enum CompiledPassBuilder<'frame> {
    Raster(CompiledRasterPassBuilder<'frame>)
}