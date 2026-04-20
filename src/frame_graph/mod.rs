mod pass;
use std::marker::PhantomData;

pub use pass::*;

mod ops;
pub use ops::*;

mod persistent;
pub use persistent::*;

pub struct StorageBuffer;
pub struct StorageTexture;
pub struct TransientTexture;
pub struct TransientBuffer;
pub struct TransientStorageTexture;
pub struct TransientStorageBuffer;
pub struct TemporalStorageTexture;
pub struct TemporalStorageBuffer;
