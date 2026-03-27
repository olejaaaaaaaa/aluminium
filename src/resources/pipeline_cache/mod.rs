mod source;
pub use source::Source;

mod raster_pipeline;
pub use raster_pipeline::{RasterPipeline, RasterPipelineDesc};

mod bindings;
pub use bindings::*;

use crate::resources::Pool;

pub struct PipelineCache {
    raster_pipelines: Pool<RasterPipeline>,
}

impl PipelineCache {
    pub fn new() -> Self {
        Self {
            raster_pipelines: Pool::new(),
        }
    }
}
