#![allow(missing_docs)]

use std::marker::PhantomData;
use std::mem::MaybeUninit;
use std::ptr::NonNull;
use std::sync::Arc;

use parking_lot::Mutex;

use super::PassContext;
use crate::frame_graph::temporal::{Id, TemporalFrameGraphResources};
use crate::frame_graph::{Create, Pass, Read, RenderTarget, Write};
use crate::resources::Destroy;
use crate::{FrameGraphTexture, Handle, Res, Resolution, TextureFormat};

pub struct RasterPass<'frame> {
    pub(crate) name: String,
    pub(crate) setup: Box<dyn for<'a> FnOnce(&mut PassBuilder<'a>) + Send + 'frame>,
    pub(crate) execute: Box<dyn FnOnce(&mut PassContext) + Send + 'frame>,
}

impl<'frame> RasterPass<'frame> {

    pub fn new<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            setup: Box::new(|_| {}),
            execute: Box::new(|_|{})
        }
    }

    pub fn setup<F, T>(mut self, closure: F) -> Self 
    where 
        F: FnOnce(&mut PassBuilder<'_>) -> T + Send + 'frame,
        T: Copy
    {
        self.setup = Box::new(|_| {});
        self
    }

    pub fn execute<F>(mut self, closure: F) -> Self 
    where 
        F: FnOnce(&mut PassContext, &()) + Send + 'frame
    {
        self.execute = Box::new(|ctx| {
            closure(ctx, &());
        });
        self
    }
}

pub struct PassBuilder<'a> {
    pub resources: &'a mut TemporalFrameGraphResources,
    pub render_target: Option<RenderTarget>,
}

impl<'a> PassBuilder<'a> {
    pub fn backbuffer(&self) -> Handle<FrameGraphTexture> {
        Handle {
            id: Id::default(),
            _marker: PhantomData,
        }
    }

    pub fn import<T: Destroy>(&mut self, handle: &Res<T>) -> Handle<T> {
        Handle {
            id: Id::default(),
            _marker: PhantomData,
        }
    }

    pub fn read<T: Read>(&mut self, handle: Handle<T>) {
        T::read(self, handle);
    }

    pub fn write<T: Write>(&mut self, handle: Handle<T>) {
        T::write(self, handle);
    }

    pub fn get_or_create_temporal<T>(&mut self, name: &'static str, desc: T) -> Handle<T> {
        Handle {
            id: Id::default(),
            _marker: PhantomData,
        }
    }

    pub fn create_texture(&mut self, name: &'static str, format: TextureFormat, resolution: Resolution) -> Handle<FrameGraphTexture> {
        let id = self.resources.textures.insert(crate::FrameGraphTextureDesc { name, format, resolution });
        Handle { 
            id, 
            _marker: PhantomData 
        }
    }

    pub fn create<T: Create>(&mut self, desc: T::Desc) -> Handle<T> {
        T::create(self.resources, desc)
    }

    pub fn uniform<T>(&mut self, desc: T) -> Handle<T> {
        Handle { 
            id: Id::default(), 
            _marker: PhantomData 
        }
    }

    pub fn transient<T>(&mut self, desc: T) -> Handle<T> {
        Handle { 
            id: Id::default(), 
            _marker: PhantomData 
        }
    }

    pub fn temporal<T>(&mut self, desc: T) -> Handle<T> {
        Handle { 
            id: Id::default(), 
            _marker: PhantomData 
        }
    }

    pub fn storage<T>(&mut self, desc: T) -> Handle<T> {
        Handle { 
            id: Id::default(), 
            _marker: PhantomData 
        }
    }
}

impl<'a> Into<Pass<'a>> for RasterPass<'a> {
    fn into(self) -> Pass<'a> {
        Pass::Raster(self)
    }
}
