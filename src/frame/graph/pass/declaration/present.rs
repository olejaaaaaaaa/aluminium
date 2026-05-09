pub struct PresentPass {
    pub(crate) name: String,
    // pub(crate) data: Box<dyn Any>,
    // pub(crate) read_textures: Vec<(Handle<TransientTexture>, Location)>,
    // pub(crate) read_storage_buffers: Vec<(*const Res<StorageBuffer>, Location)>,
    // pub(crate) setup: Option<Box<dyn for<'a> FnOnce(&mut PassBuilder<'a>) -> Box<dyn Any> + Send + 'frame>>,
    // pub(crate) execute: Option<Box<dyn FnOnce(&mut PassContext) + Send + 'frame>>,
    // pub(crate) sync: Option<Box<dyn FnOnce(vk::CommandBuffer) + 'static>>,
}

impl PresentPass {
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self { name: name.into() }
    }

    pub fn setup() {

    }

    pub fn execute() {

    }
}

pub struct PresentBuilder {

}

impl PresentBuilder {
    pub fn import() {

    }

    pub fn read() {

    }
}

pub struct PresentContext {

}

impl PresentContext {
    pub fn begin_rendering() {

    }

    pub fn draw_fullscreen() {

    }

    pub fn bind_pipeline() {

    }
}