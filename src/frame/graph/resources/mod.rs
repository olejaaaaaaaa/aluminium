use crate::{Handle, TransientTexture, resources::Res};

type ImageIndex = u32;

pub struct FrameGraphResources {
    // pub transient_textures: Vec<(TransientTextureDesc, Res<TransientTexture>)>,
    // pub transient_textures_resolve: HashMap<Id, usize>
}

struct FrameSlot<T> {
    data: T,
    last_index: u32
}

impl FrameGraphResources {
    pub fn new() -> Self {
        Self {
            // transient_textures: vec![],
            // transient_textures_resolve: HashMap::new()
        }
    }

    fn prepare_frame_data(&mut self, image_index: u32) {

    }

    fn transient_texture(&mut self, handle: Handle<TransientTexture>) -> Res<TransientTexture> {
        todo!()
    }
}
