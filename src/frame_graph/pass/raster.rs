#![allow(missing_docs)]

use super::PassContext;
use crate::{DrawCallback, frame_graph::Handle, resources};

pub struct RasterPass {
    pub(crate) name: String,
    pub(crate) texture_reads: Vec<Handle<bool>>,
    pub(crate) colot_attachment_writes: Vec<Handle<bool>>,
    pub(crate) depth_attachment_write: Option<Handle<bool>>,
    pub(crate) callback: DrawCallback,
}

impl RasterPass {
    fn new(name: String) -> Self {
        Self {
            name,
            texture_reads: vec![],
            colot_attachment_writes: vec![],
            depth_attachment_write: None,
            callback: DrawCallback::empty(),
        }
    }

    fn color_attachment(mut self, handle: Handle<bool>) -> Self {
        self.colot_attachment_writes.push(handle);
        self
    }

    fn execute(mut self, callback: DrawCallback) -> Self {
        self.callback = callback;
        self
    }
}
