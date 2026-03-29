#![allow(missing_docs)]

use bytemuck::{Pod, Zeroable};

use super::PassContext;

pub struct PresentPass {
    pub(crate) name: String,
    pub(crate) reads: Vec<bool>,
    pub(crate) constants_data: Vec<u8>,
    pub(crate) callback: Box<dyn FnOnce(&PassContext) + Send + 'static>,
}

impl PresentPass {
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            reads: vec![],
            constants_data: Vec::new(),
            callback: Box::new(|_| {}),
        }
    }

    pub fn constants<T: Pod + Zeroable>(mut self, value: T) -> Self {
        if size_of_val(&value) > 64 {
            panic!("The maximum size of Push Constants is 64 bytes")
        }
        self.constants_data = bytemuck::bytes_of(&value).to_vec();
        self
    }

    pub fn execute(mut self, callback: impl FnOnce(&PassContext) + Send + 'static) -> Self {
        self.callback = Box::new(callback);
        self
    }
}

impl Into<super::Pass> for PresentPass {
    fn into(self) -> super::Pass {
        super::Pass::Present(self)
    }
}
