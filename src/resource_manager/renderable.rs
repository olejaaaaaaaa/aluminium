use slotmap::{new_key_type, SlotMap};

use crate::resource_manager::{MaterialHandle, MeshHandle, TransformHandle};

new_key_type! {
    pub struct RenderableHandle;
}

pub struct RenderableCollection {
    data: SlotMap<RenderableHandle, Renderable>,
}

impl RenderableCollection {
    pub fn new() -> Self {
        Self {
            data: SlotMap::with_key(),
        }
    }

    pub fn add_renderable(&mut self, renderable: Renderable) -> RenderableHandle {
        self.data.insert(renderable)
    }
}

/// Renderable Object
pub struct Renderable {
    #[allow(dead_code)]
    material: MaterialHandle,
    #[allow(dead_code)]
    transform: TransformHandle,
    #[allow(dead_code)]
    mesh: MeshHandle,
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
