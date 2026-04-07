mod handle;
use std::marker::PhantomData;

pub use handle::Handle;

use crate::resources::{Destroy, Res};

enum AnyFrameGraphResource<'frame, T: Destroy> {
    External(&'frame Res<T>),
    Internal(Handle<T>),
}

pub struct FrameGraphResources {

}
