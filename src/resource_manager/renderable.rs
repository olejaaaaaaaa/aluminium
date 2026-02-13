use slotmap::{new_key_type, SlotMap};

use crate::resource_manager::{MaterialHandle, MeshHandle, TransformHandle};

new_key_type! {
    pub struct RenderableHandle;
}

pub struct RenderableCollection {
    pub data: SlotMap<RenderableHandle, Renderable>,
}

impl RenderableCollection {
    pub fn new() -> Self {
        Self {
            data: SlotMap::with_key(),
        }
    }

    pub fn get_renderables(&self) -> Vec<Renderable> {
        let mut v = vec![];
        for i in &self.data {
            v.push(*i.1);
        }
        v
    }

    pub fn add_renderable(&mut self, renderable: Renderable) -> RenderableHandle {
        self.data.insert(renderable)
    }
}

/// Renderable Object
#[derive(Clone, Copy)]
pub struct Renderable {
    #[allow(dead_code)]
    pub material: MaterialHandle,
    #[allow(dead_code)]
    pub transform: TransformHandle,
    #[allow(dead_code)]
    pub mesh: MeshHandle,
}

impl Renderable {
    /// Create new Renderable Object
    pub fn new(mesh: MeshHandle, material: MaterialHandle, transform: TransformHandle) -> Self {
        Self {
            mesh,
            material,
            transform,
        }
    }
}
