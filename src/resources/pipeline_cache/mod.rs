mod source;
pub use source::Source;

mod raster_pipeline;
pub use raster_pipeline::{RasterPipeline, RasterPipelineDesc, VertexInput};

mod bindings;
pub use bindings::*;

use crate::resources::Pool;

pub struct PipelineCache {
    pub raster_pipelines: Pool<RasterPipeline>,
}

impl PipelineCache {
    pub fn new() -> Self {
        Self {
            raster_pipelines: Pool::new(),
        }
    }
}
