#![allow(missing_docs)]

use super::{Execute, PassContext, Source};
use crate::render_graph::{PassDesc, RenderGraphResource, TextureHandle};
use crate::resource_manager::{
    FrameBufferHandle, PipelineLayoutHandle, RasterPipelineHandle, Renderable,
};

#[derive(Eq, Hash, PartialEq, Clone)]
pub struct RasterPipelineDesc {
    pub(crate) vertex_shader: Source,
    pub(crate) fragment_shader: Source,
    pub(crate) dynamic_viewport: bool,
    pub(crate) dynamic_scissors: bool,
    pub(crate) use_cache: bool,
    pub(crate) depth_test: bool,
}

impl Default for RasterPipelineDesc {
    fn default() -> Self {
        Self {
            vertex_shader: Source::None,
            fragment_shader: Source::None,
            dynamic_scissors: false,
            dynamic_viewport: false,
            use_cache: false,
            depth_test: true,
        }
    }
}

pub struct RasterPassDesc {
    pub(crate) execute_fn: Box<Execute>,
    pub(crate) writes: Vec<RenderGraphResource>,
    pub(crate) reads: Vec<RenderGraphResource>,
    pub(crate) pipeline_desc: RasterPipelineDesc,
}

impl Default for RasterPassDesc {
    fn default() -> Self {
        Self {
            writes: vec![],
            reads: vec![],
            execute_fn: Box::new(|_, _| {}),
            pipeline_desc: RasterPipelineDesc::default(),
        }
    }
}

pub struct RasterPipeline {
    pub(crate) pipeline_layout: PipelineLayoutHandle,
    pub(crate) pipeline: RasterPipelineHandle,
    pub(crate) frame_buffer: FrameBufferHandle,
}

pub struct RasterPassBuilder {
    pub(crate) inner: RasterPassDesc,
}

impl Default for RasterPassBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl RasterPassBuilder {
    pub fn new() -> Self {
        Self {
            inner: RasterPassDesc::default(),
        }
    }

    pub fn write_texture(mut self, handle: TextureHandle) -> Self {
        self.inner.writes.push(RenderGraphResource::Texture {
            handle,
            last_access: vk_sync::AccessType::Nothing,
        });
        self
    }

    // pub fn read<T: Into<RenderGraphResource>>(mut self, res: T) -> Self {
    //     self.inner.reads.push(res.into());
    //     self
    // }

    // pub fn write<T: Into<RenderGraphResource>>(mut self, res: T) -> Self {
    //     self.inner.writes.push(res.into());
    //     self
    // }

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

    pub fn execute<F>(mut self, clojure: F) -> Self
    where
        F: Fn(&PassContext, &[Renderable]) + 'static,
    {
        self.inner.execute_fn = Box::new(clojure);
        self
    }

    pub fn build(self) -> RasterPassDesc {
        self.inner
    }
}

pub struct RasterPass {
    pub(crate) pipeline: RasterPipelineHandle,
    pub(crate) layout: PipelineLayoutHandle,
    pub(crate) execute_fn: Box<Execute>,
}

impl From<RasterPassDesc> for PassDesc {
    fn from(val: RasterPassDesc) -> Self {
        PassDesc::Raster(val)
    }
}
