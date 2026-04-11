mod source;
pub use source::Source;

mod raster_pipeline;
pub use raster_pipeline::{RasterPipeline, RasterPipelineDesc, VertexInput};

mod bindings;
pub use bindings::*;

use crate::{core::PipelineLayout, resources::{Destroy, Pool, ResourceKey}};


impl Destroy for PipelineLayout {
    fn destroy(key: ResourceKey, ctx: std::sync::Weak<crate::render_context::RenderContext>, resources: std::sync::Weak<super::Resources>) {
        
    }
}

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
