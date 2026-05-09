#![allow(missing_docs)]
use std::any::Any;

use ash::vk;

use crate::{
    Handle, Location, Pass, PassBuilder, PassContext, RenderTarget, Res, StorageBuffer,
    TransientTexture,
};

pub struct RasterPass<'frame> {
    pub(crate) name: String,
    pub(crate) data: Box<dyn Any>,
    pub(crate) set: Option<vk::DescriptorSet>,
    pub(crate) render_target: RenderTarget,
    pub(crate) write_textures: Vec<Handle<TransientTexture>>,
    pub(crate) read_textures: Vec<(Handle<TransientTexture>, Location)>,
    pub(crate) read_storage_buffers: Vec<(*const Res<StorageBuffer>, Location)>,
    pub(crate) setup:
        Option<Box<dyn for<'a> FnOnce(&mut PassBuilder<'a>) -> Box<dyn Any> + Send + 'frame>>,
    pub(crate) execute: Option<Box<dyn FnOnce(&mut PassContext) + Send + 'frame>>,
    pub(crate) sync: Option<Box<dyn FnOnce(vk::CommandBuffer) + 'static>>,
}

impl<'frame> RasterPass<'frame> {
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            set: None,
            render_target: RenderTarget {
                colors: vec![],
                depth: None,
            },
            read_storage_buffers: vec![],
            write_textures: vec![],
            read_textures: vec![],
            data: Box::new(()),
            sync: None,
            setup: None,
            execute: None,
        }
    }

    pub fn setup<F, T>(mut self, f: F) -> Self
    where
        F: FnOnce(&mut PassBuilder<'_>) -> T + Send + 'frame,
        T: Copy + Send + 'static,
    {
        self.setup = Some(Box::new(move |setup| Box::new(f(setup)) as Box<dyn Any>));
        self
    }

    pub fn execute<F>(mut self, f: F) -> Self
    where
        F: FnOnce(&mut PassContext) + Send + 'frame,
    {
        self.execute = Some(Box::new(move |ctx| {
            f(ctx);
        }));
        self
    }
}

impl<'a> Into<Pass<'a>> for RasterPass<'a> {
    fn into(self) -> Pass<'a> {
        Pass::Raster(self)
    }
}
