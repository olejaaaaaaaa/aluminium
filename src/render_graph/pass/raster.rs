use std::path::PathBuf;
use std::str::FromStr;

use ash::vk;
use bytemuck::{Pod, Zeroable};

use super::{Execute, LoadOp, PassContext, StoreOp};
use crate::core::VulkanResult;
use crate::reflection::PipelineReflection;
use crate::render_graph::pass::PushConstant;
use crate::render_graph::{RenderGraphResource, TextureHandle};
use crate::resource_manager::Renderable;

pub struct RasterPipeline {
    pub(crate) push_constant: Option<Box<dyn PushConstant>>,
    pub(crate) vertex_shader: PathBuf,
    pub(crate) fragment_shader: PathBuf,
    pub(crate) use_cache: bool,
    pub(crate) depth_test: bool,
}

impl RasterPipeline {
    pub(crate) fn new() -> Self {
        Self {
            push_constant: None,
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
    pub fn push_constant(mut self, push: impl PushConstant + 'static) -> Self {
        self.pipeline.push_constant = Some(Box::new(push));
        self
    }

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
    pub(crate) execute: Box<Execute>,
    pub(crate) reflection: Option<PipelineReflection>,
}

impl RasterPass {
    pub fn new() -> Self {
        Self {
            writes: vec![],
            reads: vec![],
            pipeline: None,
            execute: Box::new(|_, _| Ok(())),
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
        F: Fn(&PassContext, &[Renderable]) -> VulkanResult<()> + 'static,
    {
        self.execute = Box::new(clojure);
        self
    }
}
