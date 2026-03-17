#![allow(missing_docs)]

use bytemuck::{Pod, Zeroable};

use super::PassContext;
use crate::frame_graph::DrawCallback;

pub struct PresentPass {
    pub(crate) name: String,
    pub(crate) reads: Vec<bool>,
    pub(crate) constants_data: Vec<u8>,
    pub(crate) callback: DrawCallback,
}

impl PresentPass {
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            reads: vec![],
            constants_data: Vec::new(),
            callback: DrawCallback::empty(),
        }
    }

    pub fn constants<T: Pod + Zeroable>(mut self, value: T) -> Self {
        self.constants_data = bytemuck::bytes_of(&value).to_vec();
        self
    }

    pub fn execute(mut self, callback: DrawCallback) -> Self {
        self.callback = callback;
        self
    }
}

impl Into<super::Pass> for PresentPass {
    fn into(self) -> super::Pass {
        super::Pass::Present(self)
    }
}
