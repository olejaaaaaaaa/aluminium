use std::marker::PhantomData;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Weak};

use super::ResourceKey;
use crate::render_context::RenderContext;
use crate::resources::Resources;

/// Unique identifier of the resource with delayed deletion
pub struct Res<T> {
    pub(crate) key: ResourceKey,
    pub(crate) ref_count: Arc<AtomicUsize>,
    pub(crate) ctx: Weak<RenderContext>,
    pub(crate) resources: Arc<Resources>,
    pub(crate) _marker: PhantomData<T>,
}

impl<T> Clone for Res<T> {
    fn clone(&self) -> Self {
        self.ref_count.fetch_add(1, Ordering::Release);
        Self {
            key: self.key,
            ref_count: self.ref_count.clone(),
            ctx: self.ctx.clone(),
            resources: self.resources.clone(),
            _marker: PhantomData,
        }
    }
}
