use std::marker::PhantomData;

use slotmap::{new_key_type, SlotMap};

new_key_type! {
    pub struct ResourceKey;
}

pub struct Pool<T> {
    map: SlotMap<ResourceKey, T>,
}

pub struct Handle<T> {
    key: ResourceKey,
    _marker: PhantomData<T>,
}

impl<T> Pool<T> {
    pub fn new() -> Self {
        Self {
            map: SlotMap::with_key(),
        }
    }

    pub fn insert(&mut self, data: T) -> Handle<T> {
        let key = self.map.insert(data);

        Handle {
            key,
            _marker: PhantomData,
        }
    }

    pub fn get(&self, handle: Handle<T>) -> Option<&T> {
        self.map.get(handle.key)
    }

    pub fn get_mut(&mut self, handle: Handle<T>) -> Option<&mut T> {
        self.map.get_mut(handle.key)
    }
}
