#![allow(missing_docs)]


use super::{Execute, PassContext, Source};
use crate::Renderable;
use crate::reflection::PipelineShaderReflection;
use crate::render_graph::{PassDesc, RenderGraphResource};
use crate::resource_manager::{PipelineLayoutHandle, RasterPipelineHandle};

pub struct PresentPassDesc {
    pub(crate) reads: Vec<RenderGraphResource>,
    pub(crate) execute_fn: Box<Execute>,
    pub(crate) vertex_shader: Source,
    pub(crate) fragment_shader: Source,
    pub(crate) use_cache: bool,
    pub(crate) depth_test: bool,
}

impl Default for PresentPassDesc {
    fn default() -> Self {
        Self { 
            reads: vec![], 
            execute_fn: Box::new(|_, _| {}), 
            vertex_shader: Source::None, 
            fragment_shader: Source::None, 
            use_cache: false, 
            depth_test: true 
        }
    }
}

pub struct PresentPassBuilder {
    inner: PresentPassDesc
}

impl PresentPassBuilder {

    pub fn new() -> Self {
        Self { inner: PresentPassDesc::default() }
    }

    pub fn vertex(mut self, shader: impl Into<Source>) -> Self {
        self.inner.vertex_shader = shader.into();
        self
    }

    pub fn fragment(mut self, shader: impl Into<Source>) -> Self {
        self.inner.fragment_shader = shader.into();
        self
    }

    pub fn depth_test(mut self, enable: bool) -> Self {
        self.inner.depth_test = enable;
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
    pub(crate) reflection: PipelineShaderReflection,
    pub(crate) reads: Vec<RenderGraphResource>,
    pub(crate) pipeline_layout: PipelineLayoutHandle,
    pub(crate) pipeline: RasterPipelineHandle,
    pub(crate) execute_fn: Box<Execute>,
}

impl Into<PassDesc> for PresentPassDesc {
    fn into(self) -> PassDesc {
        PassDesc::Present(self)
    }
}
