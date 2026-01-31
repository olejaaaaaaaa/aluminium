use crate::resource_manager::{MaterialHandle, MeshHandle, TransformHandle};

#[derive(Clone, Copy)]
pub struct RenderableHandle(pub usize);

pub struct RenderableCollection {
    data: Vec<Renderable>,
}

impl RenderableCollection {
    pub fn new() -> Self {
        Self { data: vec![] }
    }

    pub fn add_renderable(&mut self, renderable: Renderable) -> RenderableHandle {
        let index = self.data.len();
        self.data.push(renderable);
        RenderableHandle(index)
    }
}

pub struct Renderable {
    material: MaterialHandle,
    transform: TransformHandle,
    mesh: MeshHandle,
}

impl Renderable {
    pub fn new(mesh: MeshHandle, material: MaterialHandle, transform: TransformHandle) -> Self {
        Self {
            mesh,
            material,
            transform,
        }
    }
}
