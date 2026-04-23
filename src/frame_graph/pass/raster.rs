#![allow(missing_docs)]

use std::any::Any;
use std::marker::PhantomData;
use std::mem::MaybeUninit;
use std::ops::Add;
use std::ptr::NonNull;
use std::sync::Arc;

use parking_lot::Mutex;

use super::PassContext;
use crate::frame_graph::{LoadOp, Pass, RenderTarget, StoreOp, TransientTexture};
use crate::frame_scope::{Id, TemporalFrameGraphResources, TransientTextureDesc};
use crate::resources::Destroy;
use crate::{FrameGraphTexture, Handle, Res, Resolution, TextureFormat};

pub struct RasterPass<'frame> {
    pub(crate) name: String,
    pub(crate) data: Box<dyn Any>,
    pub(crate) setup: Option<Box<dyn for<'a> FnOnce(&mut PassBuilder<'a>) -> Box<dyn Any> + Send + 'frame>>,
    pub(crate) execute: Box<dyn FnOnce(&mut PassContext) + Send + 'frame>,
}

impl<'frame> RasterPass<'frame> {
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            data: Box::new(()),
            setup: None,
            execute: Box::new(|_| {}),
        }
    }

    pub fn setup<F, T>(mut self, f: F) -> Self
    where
        F: FnOnce(&mut PassBuilder<'_>) -> T + Send + 'frame,
        T: Copy + Send + 'static,
    {
        self.setup = Some(Box::new(move |setup| {
            Box::new(f(setup)) as Box<dyn Any>
        }));
        self
    }

    pub fn execute<F>(mut self, f: F) -> Self
    where
        F: FnOnce(&mut PassContext) + Send + 'frame,
    {
        self.execute = Box::new(move |ctx| {
            f(ctx);
        });
        self
    }
}

pub struct Location {
    pub set: u32,
    pub binding: u32
}

pub struct PassBuilder<'a> {
    pub(crate) resources: &'a mut TemporalFrameGraphResources,
}

impl<'a> PassBuilder<'a> {
    pub fn backbuffer(&self) -> Handle<TransientTexture> {
        Handle {
            id: Id::default(),
            version: 0,
            _marker: PhantomData,
        }
    }

    pub fn write_depth_stencil(&mut self, texture: Handle<TransientTexture>, stencil_load: LoadOp, stencil_store: StoreOp) -> Handle<TransientTexture> {
        Handle { 
            id: texture.id, 
            version: texture.version.add(1), 
            _marker: PhantomData
        }
    }

    pub fn write_depth(&mut self, texture: Handle<TransientTexture>, load: LoadOp, store: StoreOp) -> Handle<TransientTexture> {
        Handle { 
            id: texture.id, 
            version: texture.version.add(1), 
            _marker: PhantomData
        }
    }

    pub fn read_texture(&mut self, texture: Handle<TransientTexture>, location: Location) -> Handle<TransientTexture> {
        Handle::default()
    }
 
    pub fn write_color(&mut self, texture: Handle<TransientTexture>, load: LoadOp, store: StoreOp) -> Handle<TransientTexture> {
        Handle { 
            id: texture.id, 
            version: texture.version.add(1), 
            _marker: PhantomData
        }
    }

    pub fn create_storage_texture(&mut self, name: &'static str, format: TextureFormat, resolution: Resolution) -> Handle<TransientTexture> {
        Handle::default()
    }

    pub fn create_storage_buffer(&mut self, name: &'static str, size: u64) -> Handle<TransientTexture> {
        Handle::default()
    }

    pub fn get_or_create_temporal_texture(&mut self, name: &'static str, format: TextureFormat, resolution: Resolution) -> Handle<TransientTexture> {
        Handle::default()
    }

    pub fn get_or_create_temporal_buffer(&mut self, name: &'static str, size: u64) -> Handle<TransientTexture> {
        Handle::default()
    }

    pub fn create_buffer(&mut self, name: &'static str, size: u64) -> Handle<TransientTexture> {
        Handle::default()
    }

    pub fn create_texture(
        &mut self,
        name: &'static str,
        format: TextureFormat,
        resolution: Resolution,
    ) -> Handle<TransientTexture> {

        let id = self.resources.textures.insert(TransientTextureDesc {
            name,
            format,
            resolution,
        });

        Handle {
            id,
            version: 0,
            _marker: PhantomData,
        }
    }
}

impl<'a> Into<Pass<'a>> for RasterPass<'a> {
    fn into(self) -> Pass<'a> {
        Pass::Raster(self)
    }
}
