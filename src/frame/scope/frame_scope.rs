use crate::{Pass, PassBuilder, RenderTarget, frame::scope::FrameResources};

pub struct FrameScope<'frame> {
    pub(crate) resources: FrameResources,
    pub(crate) passes: Vec<Pass<'frame>>,
    pub(crate) execution_order: Vec<usize>,
}

impl<'frame> FrameScope<'frame> {
    pub fn new() -> Self {
        Self {
            passes: vec![],
            resources: FrameResources::new(),
            execution_order: vec![],
        }
    }

    pub fn add_pass<P: Into<Pass<'frame>>, T: Clone + 'static>(&mut self, value: P) -> T {

        match value.into() {
            Pass::Raster(mut pass) => {
                
                if let Some(setup) = pass.setup.take() {

                    let mut builder = PassBuilder {
                        read_storage_buffers: vec![],
                        write_textures: vec![],
                        read_textures: vec![],
                        render_target: RenderTarget {
                            colors: vec![],
                            depth: None
                        },
                        resources: &mut self.resources
                    };

                    let data = (setup)(&mut builder);
                    pass.data = data;
                    pass.read_storage_buffers = builder.read_storage_buffers;
                    pass.render_target = builder.render_target;
                    pass.write_textures = builder.write_textures;
                    pass.read_textures = builder.read_textures;
                }
                
                let r = pass.data.downcast_ref::<T>().cloned().expect("AAAAA");
                self.passes.push(Pass::Raster(pass));
                return r;
            }
        }
    }
}
