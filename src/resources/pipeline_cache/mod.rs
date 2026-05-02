mod source;
pub use source::Source;

mod raster_pipeline;
pub use raster_pipeline::{RasterPipeline, RasterPipelineDesc, VertexInput, Layout};

mod bindings;
pub use bindings::*;

use crate::{core::PipelineLayout, resources::Pool};

pub struct PipelineCache {
    pub pipeline_layout: Pool<PipelineLayout>,
    pub raster_pipelines: Pool<RasterPipeline>,
}

impl PipelineCache {
    pub fn new() -> Self {
        Self {
            pipeline_layout: Pool::new(),
            raster_pipelines: Pool::new(),
        }
    }
}
