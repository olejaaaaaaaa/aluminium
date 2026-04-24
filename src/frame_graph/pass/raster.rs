#![allow(missing_docs)]

use std::any::Any;
use std::marker::PhantomData;
use std::mem::MaybeUninit;
use std::ops::Add;
use std::ptr::NonNull;
use std::sync::Arc;

use parking_lot::Mutex;

use super::PassContext;
use crate::frame_graph::{LoadOp, Pass, PassBuilder, StoreOp};
use crate::frame_scope::{Id, TransientTextureDesc};
use crate::resources::Destroy;
use crate::{RenderTarget, Res, Resolution, TextureFormat};

pub struct RasterPass<'frame> {
    pub(crate) name: String,
    pub(crate) data: Box<dyn Any>,
    pub(crate) render_target: RenderTarget,
    pub(crate) setup: Option<Box<dyn for<'a> FnOnce(&mut PassBuilder<'a>) -> Box<dyn Any> + Send + 'frame>>,
    pub(crate) execute: Box<dyn FnOnce(&mut PassContext) + Send + 'frame>,
}

impl<'frame> RasterPass<'frame> {
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            render_target: RenderTarget { colors: vec![], depth: None },
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



impl<'a> Into<Pass<'a>> for RasterPass<'a> {
    fn into(self) -> Pass<'a> {
        Pass::Raster(self)
    }
}
