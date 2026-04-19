mod pass;
use std::marker::PhantomData;

pub use pass::*;

pub mod temporal;
use temporal::TemporalFrameGraphResources;

mod persistent;
pub use persistent::*;

use crate::{Res, frame_graph::temporal::Id, resources::Destroy};


struct StorageBuffer;
struct StorageTexture;
struct TransientTexture;
struct TransientBuffer;
struct TemporalStorageTexture;
struct TemporalStorageBuffer;


trait Read: Sized {
    fn read(builder: &mut PassBuilder<'_>, handle: Handle<Self>);
}

impl Read for FrameGraphTexture {
    fn read(builder: &mut PassBuilder<'_>, handle: Handle<Self>) {
        
    }
}

trait Write: Sized {
    fn write(builder: &mut PassBuilder<'_>, handle: Handle<Self>);
}

impl Write for FrameGraphTexture {
    fn write(builder: &mut PassBuilder<'_>, handle: Handle<Self>) {
        
    }
}

trait Import {
    type Output;
    fn import(resources: &mut TemporalFrameGraphResources, handle: &Res<Self::Output>) where <Self as Import>::Output: Destroy;
}

pub trait Create: Sized {
    type Desc;
    fn create(resources: &mut TemporalFrameGraphResources, desc: Self::Desc) -> Handle<Self>;
}

impl Create for FrameGraphTexture {
    type Desc = FrameGraphTextureDesc;
    fn create(resources: &mut TemporalFrameGraphResources, desc: Self::Desc) -> Handle<Self> {
        let id = resources.textures.insert(desc);
        Handle { 
            id, 
            _marker: PhantomData 
        }
    }
}

impl Create for FrameGraphUniform {
    type Desc = FrameGraphUniformDesc;
    fn create(resources: &mut TemporalFrameGraphResources, desc: Self::Desc) -> Handle<Self> {
        Handle { 
            id: Id::default(), 
            _marker: PhantomData 
        }
    }
}
