use std::marker::PhantomData;

use slotmap::{new_key_type, SlotMap};
use std::sync::{Arc, Mutex};

new_key_type! {
    pub struct ResourceKey;
}

struct ResourceEntry<T> {
    data: T,
}

type SharedMap<T> = Arc<Mutex<SlotMap<ResourceKey, ResourceEntry<T>>>>;

struct DropGuard<T> {
    key: ResourceKey,
    map: SharedMap<T>,
}

impl<T> Drop for DropGuard<T> {
    fn drop(&mut self) {
        let mut map = self.map.lock().unwrap();
        map.remove(self.key);
    }
}

pub struct Pool<T> {
    map: SharedMap<T>,
}

pub struct Handle<T> {
    key: ResourceKey,
    _guard: Arc<DropGuard<T>>,
    _marker: PhantomData<T>,
}

impl<T> Clone for Handle<T> {
    fn clone(&self) -> Self {
        Handle {
            key: self.key,
            _guard: Arc::clone(&self._guard),
            _marker: PhantomData,
        }
    }
}

impl<T> Pool<T> {
    pub fn new() -> Self {
        Self {
            map: Arc::new(Mutex::new(SlotMap::with_key())),
        }
    }

    pub fn insert(&mut self, data: T) -> Handle<T> {
        let key = self.map.lock().unwrap().insert(ResourceEntry { data });

        Handle {
            key,
            _guard: Arc::new(DropGuard {
                key,
                map: Arc::clone(&self.map),
            }),
            _marker: PhantomData,
        }
    }

    pub fn get(&self, handle: Handle<T>) -> Option<&T> {
        // Проблема: не можем вернуть &T из-за Mutex — lifetime не пробросить
        // Решение: используем unsafe с явной гарантией что Pool живёт дольше &T
        let map = self.map.lock().unwrap();
        let ptr = &map.get(handle.key)?.data as *const T;
        // SAFETY: данные живут пока Pool жив, а Pool живёт дольше любого Handle
        Some(unsafe { &*ptr })
    }

    pub fn get_mut(&mut self, handle: Handle<T>) -> Option<&mut T> {
        let mut map = self.map.lock().unwrap();
        let ptr = &mut map.get_mut(handle.key)?.data as *mut T;
        // SAFETY: &mut Pool гарантирует эксклюзивный доступ
        Some(unsafe { &mut *ptr })
    }
}