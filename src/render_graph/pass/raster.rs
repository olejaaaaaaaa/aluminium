#![allow(missing_docs)]

use std::path::PathBuf;

use slotmap::Key;

use super::{Execute, LoadOp, PassContext, Source, StoreOp};
use crate::reflection::PipelineShaderReflection;
use crate::render_graph::{RenderGraphResource, TextureHandle};
use crate::resource_manager::{
    FrameBufferHandle, PipelineLayoutHandle, RasterPipelineHandle, Renderable,
};

#[derive(Eq, Hash, PartialEq, Clone)]
pub struct RasterPipelineDesc {
    pub(crate) vertex_shader: Source,
    pub(crate) fragment_shader: Source,
    pub(crate) use_cache: bool,
    pub(crate) depth_test: bool,
}

impl Default for RasterPipelineDesc {
    fn default() -> Self {
        Self {
            vertex_shader: Source::None,
            fragment_shader: Source::None,
            use_cache: false,
            depth_test: true,
        }
    }
}

pub struct RasterPassDesc {
    pub(crate) execute_fn: Box<Execute>,
    pub(crate) writes: Vec<RenderGraphResource>,
    pub(crate) reads: Vec<RenderGraphResource>,
    pub(crate) vertex_shader: Source,
    pub(crate) fragment_shader: Source,
    pub(crate) use_cache: bool,
    pub(crate) depth_test: bool,
}

pub struct RasterPipeline {
    pub(crate) pipeline_layout: PipelineLayoutHandle,
    pub(crate) pipeline: RasterPipelineHandle,
    pub(crate) frame_buffer: FrameBufferHandle,
    pub(crate) vertex_shader: PathBuf,
    pub(crate) fragment_shader: PathBuf,
    #[allow(dead_code)]
    pub(crate) use_cache: bool,
    pub(crate) depth_test: bool,
}

impl RasterPipeline {
    pub(crate) fn new() -> Self {
        Self {
            frame_buffer: FrameBufferHandle::null(),
            pipeline_layout: PipelineLayoutHandle::null(),
            pipeline: RasterPipelineHandle::null(),
            vertex_shader: PathBuf::new(),
            fragment_shader: PathBuf::new(),
            use_cache: false,
            depth_test: true,
        }
    }
}

pub struct RasterPipelineBuilder {
    pub(crate) pass: RasterPass,
    pub(crate) pipeline: RasterPipeline,
}

impl RasterPipelineBuilder {
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

    pub fn end_pipeline(mut self) -> RasterPass {
        self.pass.pipeline = Some(self.pipeline);
        self.pass
    }
}

pub struct RasterPass {
    pub(crate) writes: Vec<RenderGraphResource>,
    pub(crate) reads: Vec<RenderGraphResource>,
    pub(crate) pipeline: Option<RasterPipeline>,
    pub(crate) execute_fn: Box<Execute>,
    pub(crate) reflection: Option<PipelineShaderReflection>,
}

impl Default for RasterPass {
    fn default() -> Self {
        Self::new()
    }
}

impl RasterPass {
    pub fn new() -> Self {
        Self {
            writes: vec![],
            reads: vec![],
            pipeline: None,
            execute_fn: Box::new(|_, _| {}),
            reflection: None,
        }
    }

    pub fn render_target(mut self, handle: TextureHandle, load: LoadOp, store: StoreOp) -> Self {
        self.writes.push(RenderGraphResource::RenderTarget {
            texture: (handle, vk_sync::AccessType::Nothing),
            ops: (load, store),
        });
        self
    }

    pub fn pipeline(self) -> RasterPipelineBuilder {
        RasterPipelineBuilder {
            pass: self,
            pipeline: RasterPipeline::new(),
        }
    }

    pub fn read<T: Into<RenderGraphResource>>(mut self, res: T) -> Self {
        self.reads.push(res.into());
        self
    }

    pub fn write<T: Into<RenderGraphResource>>(mut self, res: T) -> Self {
        self.reads.push(res.into());
        self
    }

    pub fn render<F>(mut self, clojure: F) -> Self
    where
        F: Fn(&PassContext, &[Renderable]) + 'static,
    {
        self.execute_fn = Box::new(clojure);
        self
    }
}
