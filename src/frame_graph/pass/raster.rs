#![allow(missing_docs)]

use super::PassContext;
use crate::frame_graph::Handle;

pub struct RasterPass {
    pub(crate) name: String,
    pub(crate) texture_reads: Vec<Handle<bool>>,
    pub(crate) colot_attachment_writes: Vec<Handle<bool>>,
    pub(crate) depth_attachment_write: Option<Handle<bool>>,
    pub(crate) callback: Box<dyn FnOnce(&PassContext) + Send + 'static>,
}

impl RasterPass {
    fn new(name: String) -> Self {
        Self {
            name,
            texture_reads: vec![],
            colot_attachment_writes: vec![],
            depth_attachment_write: None,
            callback: Box::new(|_| {}),
        }
    }

    fn color_attachment(mut self, handle: Handle<bool>) -> Self {
        self.colot_attachment_writes.push(handle);
        self
    }

    fn execute(mut self, callback: impl FnOnce(&PassContext) + Send + 'static) -> Self {
        self.callback = Box::new(callback);
        self
    }
}
