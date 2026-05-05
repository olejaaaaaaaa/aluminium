use std::sync::RwLockReadGuard;

use parking_lot::RwLock;
use slotmap::{new_key_type, SlotMap};

new_key_type! {
    #[allow(missing_docs)]
    pub struct ResourceKey;
}

pub struct Pool<T>(pub(crate) RwLock<SlotMap<ResourceKey, T>>);

impl<T> Pool<T> {
    pub fn new() -> Self {
        Self(RwLock::new(SlotMap::with_key()))
    }

    // Добавьте другие удобные методы
    pub fn insert(&self, value: T) -> ResourceKey {
        self.0.write().insert(value)
    }
}
