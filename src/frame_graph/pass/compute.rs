#![allow(missing_docs)]
use super::PassContext;

pub struct ComputePass<'frame> {
    pub(crate) name: String,
    pub(crate) callback: Box<dyn FnOnce(&mut PassContext) + Send + 'frame>,
}

impl<'frame> ComputePass<'frame> {
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            callback: Box::new(|_| {}),
        }
    }

    fn execute(mut self, callback: impl FnOnce(&mut PassContext) + Send + 'frame) -> Self {
        self.callback = Box::new(callback);
        self
    }
}

impl<'a> Into<super::Pass<'a>> for ComputePass<'a> {
    fn into(self) -> super::Pass<'a> {
        super::Pass::Compute(self)
    }
}

