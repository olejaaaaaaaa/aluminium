#![allow(missing_docs)]

use super::{Execute, PassContext, Source};
use crate::render_graph::{PassDesc, RasterPipelineDesc, RenderGraphResource};
use crate::resource_manager::{PipelineLayoutHandle, RasterPipelineHandle};
use crate::Renderable;

pub struct PresentPassDesc {
    pub(crate) reads: Vec<RenderGraphResource>,
    pub(crate) execute_fn: Box<Execute>,
    pub(crate) pipeline_desc: RasterPipelineDesc,
}

impl Default for PresentPassDesc {
    fn default() -> Self {
        Self {
            reads: vec![],
            execute_fn: Box::new(|_, _| {}),
            pipeline_desc: RasterPipelineDesc::default(),
        }
    }
}

pub struct PresentPassBuilder {
    inner: PresentPassDesc,
}

impl Default for PresentPassBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl PresentPassBuilder {
    pub fn new() -> Self {
        Self {
            inner: PresentPassDesc::default(),
        }
    }

    pub fn dynamic_scissors(mut self, value: bool) -> Self {
        self.inner.pipeline_desc.dynamic_scissors = value;
        self
    }

    pub fn dynamic_viewport(mut self, value: bool) -> Self {
        self.inner.pipeline_desc.dynamic_viewport = value;
        self
    }

    pub fn vertex(mut self, src: impl Into<Source>) -> Self {
        self.inner.pipeline_desc.vertex_shader = src.into();
        self
    }

    pub fn fragment(mut self, src: impl Into<Source>) -> Self {
        self.inner.pipeline_desc.fragment_shader = src.into();
        self
    }

    pub fn depth_test(mut self, enable: bool) -> Self {
        self.inner.pipeline_desc.depth_test = enable;
        self
    }

    pub fn read<T: Into<RenderGraphResource>>(mut self, res: T) -> Self {
        self.inner.reads.push(res.into());
        self
    }

    pub fn execute<F>(mut self, clojure: F) -> Self
    where
        F: Fn(&PassContext, &[Renderable]) + 'static,
    {
        self.inner.execute_fn = Box::new(clojure);
        self
    }

    pub fn build(self) -> PresentPassDesc {
        self.inner
    }
}

pub struct PresentPass {
    pub(crate) layout: PipelineLayoutHandle,
    pub(crate) pipeline: RasterPipelineHandle,
    pub(crate) execute_fn: Box<Execute>,
}

impl From<PresentPassDesc> for PassDesc {
    fn from(val: PresentPassDesc) -> Self {
        PassDesc::Present(val)
    }
}
