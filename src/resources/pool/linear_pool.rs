use std::marker::PhantomData;
use std::sync::atomic::AtomicUsize;
use std::sync::{Arc, Weak};

use slotmap::SlotMap;

use crate::render_context::RenderContext;
use crate::resources::{Destroy, Res, ResourceKey, Resources};

pub struct LinearPool<T: Destroy> {
    current_index: usize,
    slots: SlotMap<ResourceKey, usize>,
    data: Vec<T>,
    _marker: PhantomData<T>,
}

impl<T: Destroy> LinearPool<T> {
    pub fn new(size: usize) -> Self {
        Self {
            slots: SlotMap::with_key(),
            current_index: 0,
            data: Vec::with_capacity(size),
            _marker: PhantomData,
        }
    }

    pub fn insert(&mut self, ctx: Weak<RenderContext>, resources: Weak<Resources>, value: T) -> Res<T> {
        let key = self.slots.insert(self.current_index);
        self.data.push(value);
        self.current_index += 1;
        Res {
            key,
            ref_count: Arc::new(AtomicUsize::new(1)),
            ctx,
            resources,
            _marker: PhantomData,
        }
    }

    pub fn get(&self, res: &Res<T>) -> &T {
        let index = self.slots.get(res.key).expect("Resource not found");
        &self.data[*index]
    }

    pub fn get_mut(&mut self, res: &Res<T>) -> &mut T {
        let index = self.slots.get(res.key).expect("Resource not found");
        &mut self.data[*index]
    }

    pub fn index(&self, res: &Res<T>) -> usize {
        *self.slots.get(res.key).expect("Resource not found")
    }

    pub fn as_slice(&self) -> &[T] {
        &self.data
    }
}
