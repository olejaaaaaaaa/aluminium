use std::marker::PhantomData;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Weak};
use parking_lot::RwLock;
use slotmap::new_key_type;

use crate::render_context::RenderContext;
use crate::VulkanResult;

mod transform;
pub use transform::{Transform, TransformDesc, TransformPool};

mod pool;
pub use pool::{LinearPool, Pool};

mod mesh;
pub use mesh::{Mesh, MeshDesc, MeshStore};

mod pipeline_cache;
pub use pipeline_cache::*;

new_key_type! {
    #[allow(missing_docs)]
    pub struct ResourceKey;
}

/// Unique identifier of the resource with delayed deletion
pub struct Res<T: Destroy> {
    pub(crate) key: ResourceKey,
    pub(crate) ref_count: Arc<AtomicUsize>,
    pub(crate) root: Weak<Resources>,
    pub(crate) _marker: PhantomData<T>,
}

impl<T: Destroy> Clone for Res<T> {
    fn clone(&self) -> Self {
        self.ref_count.fetch_add(1, Ordering::Release);
        Self {
            key: self.key,
            ref_count: self.ref_count.clone(),
            root: self.root.clone(),
            _marker: PhantomData,
        }
    }
}

impl<T: Destroy> Drop for Res<T> {
    fn drop(&mut self) {
        let ref_count = self.ref_count.fetch_sub(1, Ordering::AcqRel);
        if ref_count == 1 {
            T::destroy(self.key, &self.root.upgrade().expect("Resources already dropped"));
        }
    }
}

#[allow(missing_docs)]
pub trait Create: Sized + Destroy {
    type Desc<'a>;
    fn create(resources: &Resources, desc: Self::Desc<'_>) -> VulkanResult<Res<Self>>;
}

#[allow(missing_docs)]
pub trait GetMut: Sized + Destroy {
    fn get_mut<'a>(resources: &'a Resources, res: &Res<Self>) -> &'a mut Self;
}

#[allow(missing_docs)]
pub trait Get: Sized + Destroy {
    fn get<'a>(resources: &'a Resources, res: &Res<Self>) -> &'a Self;
}

#[allow(missing_docs)]
pub trait Destroy {
    fn destroy(key: ResourceKey, resources: &Resources);
}

pub struct Resources {
    pub(crate) ctx: Arc<RenderContext>,
    pub(crate) transforms: RwLock<TransformPool>,
    pub(crate) mesh: RwLock<MeshStore>,
    // pub(crate) pipeline_cache: RwLock<PipelineCache>
}

impl Resources {
    pub fn new(ctx: Arc<RenderContext>) -> VulkanResult<Arc<Self>> {
        let frame_count = ctx.window.read().frame_buffers.len();
        Ok(Arc::new_cyclic(|weak| Self {
            transforms: RwLock::new(TransformPool::new(&ctx.device, frame_count, weak.clone()).expect("Error creating transform pool")),
            mesh: RwLock::new(MeshStore::new(weak.clone())),
            // pipeline_cache: RwLock::new(PipelineCache::new(weak.clone())),
            ctx,
        }))
    }
}
