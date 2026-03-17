mod linear_pool;
use std::marker::PhantomData;
use std::sync::atomic::AtomicUsize;
use std::sync::{Arc, Weak};

pub use linear_pool::LinearPool;
use slotmap::SlotMap;

use crate::resources::{Destroy, Res, ResourceKey, Resources};

pub struct Pool<T: Destroy> {
    slots: SlotMap<ResourceKey, T>,
    root: Weak<Resources>,
}

impl<T: Destroy> Pool<T> {
    pub fn new(root: Weak<Resources>) -> Self {
        Self {
            slots: SlotMap::with_key(),
            root,
        }
    }

    pub fn insert(&mut self, value: T) -> Res<T> {
        let key = self.slots.insert(value);
        Res {
            key,
            ref_count: Arc::new(AtomicUsize::new(1)),
            root: self.root.clone(),
            _marker: PhantomData,
        }
    }

    pub fn get(&self, res: &Res<T>) -> &T {
        self.slots.get(res.key).expect("Resource not found")
    }

    pub fn get_mut(&mut self, res: &Res<T>) -> &mut T {
        self.slots.get_mut(res.key).expect("Resource not found")
    }

    pub fn remove(&mut self, key: ResourceKey) {
        self.slots.remove(key);
    }
}
