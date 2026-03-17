#![allow(missing_docs)]

use super::PassContext;
use crate::{resources, DrawCallback};

pub struct RasterPass {
    pub(crate) name: String,
    pub(crate) reads: Vec<bool>,
    pub(crate) writes: Vec<bool>,
    pub(crate) callback: DrawCallback,
}

impl RasterPass {
    fn new(name: String) -> Self {
        Self {
            name,
            reads: vec![],
            writes: vec![],
            callback: DrawCallback::empty(),
        }
    }

    fn execute(mut self, callback: DrawCallback) -> Self {
        self.callback = callback;
        self
    }
}
