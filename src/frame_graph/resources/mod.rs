mod handle;
pub use handle::Handle;

use crate::resources::{Destroy, Res};

pub trait Import {}

enum GraphResource<T: Destroy> {
    External(Res<T>),
    Internal(Handle<T>),
}

pub struct FrameGraphResources {}
