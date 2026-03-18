use std::marker::PhantomData;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Weak};
use parking_lot::RwLock;
use slotmap::new_key_type;

use crate::bindless::Bindless;
use crate::camera::Camera;
use crate::core::Device;
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

pub struct Ref<'a, T>(pub(crate) parking_lot::RwLockReadGuard<'a, T>);
pub struct RefMut<'a, T>(pub(crate) parking_lot::RwLockWriteGuard<'a, T>);

impl<'a, T> std::ops::Deref for Ref<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.0
    }
}

impl<'a, T> std::ops::Deref for RefMut<'a, T> {
    type Target = T;
    fn deref(&self) -> &T { 
        &self.0
    }
}

impl<'a, T> std::ops::DerefMut for RefMut<'a, T> {
    fn deref_mut(&mut self) -> &mut T { 
        &mut self.0
    }
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
    fn get_mut<'a>(resources: &'a Resources, res: &Res<Self>) -> RefMut<'a, Self>;
}

#[allow(missing_docs)]
pub trait Get: Sized + Destroy {
    fn get<'a>(resources: &'a Resources, res: &Res<Self>) -> Ref<'a, Self>;
}

#[allow(missing_docs)]
pub trait Destroy {
    fn destroy(key: ResourceKey, resources: &Resources);
}

pub struct Resources {
   //pub(crate) bindless: Bindless,
   //pub(crate) camera: RwLock<Camera>
}

impl Resources {
    pub fn new(ctx: &Arc<RenderContext>) -> VulkanResult<Arc<Self>> {
        
        //let frame_count = ctx.frame_count();
        //let camera = Camera::new(&ctx.device, frame_count)?;
        //let bindless = Bindless::new(&ctx)?;

        Ok(Arc::new(Self {
            //bindless,
            //camera: RwLock::new(camera)
        }))
    }

    // Always Set 0
    pub fn bindless_set(&self) {

    }

    // Always Set 1
    pub fn per_frame_set(&self) {

    }

    pub(crate) fn destroy(&self, device: &Device) {
        
    }


}
