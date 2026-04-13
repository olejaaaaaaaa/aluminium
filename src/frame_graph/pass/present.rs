#![allow(missing_docs)]

use std::any::{Any, TypeId};
use std::marker::PhantomData;

use super::PassContext;
use crate::frame_graph::pass::IntoPass;
use crate::frame_graph::{Pass, RenderTarget};

pub struct PresentPass<'frame> {
    pub(crate) name: String,
    pub(crate) execute: Box<dyn FnOnce(&mut PassContext) + Send + 'frame>,
}

impl<'frame> PresentPass<'frame> {
    pub fn new<Name, Type, Setup, Execute>(name: Name, setup: Setup, execute: Execute) -> (Self, Type)
    where
        Name: Into<String>,
        Type: Any + Send + Copy + Clone + Default,
        Setup: FnOnce(&mut RasterPassBuilder<'frame>) -> Type + Send + 'frame,
        Execute: FnOnce(&mut PassContext, &Type) + Send + 'frame,
    {
        let mut builder = RasterPassBuilder::default();
        let data = setup(&mut builder);

        (
            Self {
                name: name.into(),
                execute: Box::new(move |ctx| {
                    execute(ctx, &data);
                }),
            },
            data,
        )
    }
}

#[derive(Default)]
pub struct RasterPassBuilder<'frame> {
    pub render_target: Option<RenderTarget<'frame>>,
}

impl<'frame> RasterPassBuilder<'frame> {

    pub fn backbuffer(&self) {

    }

    pub fn read(&mut self) {}

    pub fn write(&mut self) {}

    pub fn create<T>(&mut self, desc: T) {}
}


impl<'frame, T: Copy + Any + Send + 'static> IntoPass<'frame, T> for (PresentPass<'frame>, T) {
    fn into_pass(self) -> (Pass<'frame>, T) {
        (self.0.into(), self.1)
    }
}

impl<'a> Into<super::Pass<'a>> for PresentPass<'a> {
    fn into(self) -> super::Pass<'a> {
        super::Pass::Present(self)
    }
}
