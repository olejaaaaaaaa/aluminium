use std::path::PathBuf;

use ash::vk;

use super::{Execute, LoadOp, PassContext, StoreOp};
use crate::core::VulkanResult;
use crate::render_graph::pass::raster::{RasterPipeline, RasterPipelineBuilder};
use crate::render_graph::RenderGraphResource;
use crate::resource_manager::Renderable;

pub struct PresentPipelineBuilder {
    pub(crate) pass: PresentPass,
    pub(crate) pipeline: RasterPipeline,
}

impl PresentPipelineBuilder {
    pub fn vertex(mut self, shader: impl Into<PathBuf>) -> Self {
        self.pipeline.vertex_shader = shader.into();
        self
    }

    pub fn fragment(mut self, shader: impl Into<PathBuf>) -> Self {
        self.pipeline.fragment_shader = shader.into();
        self
    }

    pub fn depth_test(mut self, enable: bool) -> Self {
        self.pipeline.depth_test = enable;
        self
    }

    pub fn end_pipeline(mut self) -> PresentPass {
        self.pass.pipeline = Some(self.pipeline);
        self.pass
    }
}

pub struct PresentPass {
    pub(crate) reads: Vec<RenderGraphResource>,
    pub(crate) pipeline: Option<RasterPipeline>,
    pub(crate) execute: Box<Execute>,
}

impl PresentPass {
    pub fn new() -> Self {
        Self {
            reads: vec![],
            pipeline: None,
            execute: Box::new(|_, _| Ok(())),
        }
    }

    pub fn pipeline(self) -> PresentPipelineBuilder {
        PresentPipelineBuilder {
            pass: self,
            pipeline: RasterPipeline::new(),
        }
    }

    pub fn read<T: Into<RenderGraphResource>>(mut self, res: T) -> Self {
        self.reads.push(res.into());
        self
    }

    /// Draw to swapchain image
    pub fn draw<F>(mut self, clojure: F) -> Self
    where
        F: Fn(&PassContext, &[Renderable]) -> VulkanResult<()> + 'static,
    {
        self.execute = Box::new(clojure);
        self
    }
}
