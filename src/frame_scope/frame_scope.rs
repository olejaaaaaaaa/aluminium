use std::sync::Arc;

use parking_lot::Mutex;
use crate::{Handle, RenderTarget, TransientTexture, frame_graph::{Pass, PassBuilder}, frame_scope::resources::FrameResources, render_context::RenderContext};

pub struct FrameScope<'frame> {
    pub resources: FrameResources,
    pub passes: Vec<Pass<'frame>>,
    execution_order: Vec<usize>,
}

impl<'frame> FrameScope<'frame> {
    pub fn new() -> Self {
        Self {
            passes: vec![],
            resources: FrameResources::new(),
            execution_order: vec![],
        }
    }

    fn topological_sort(&mut self) {
        profiling::scope!("FrameGraph::topological_sort");
        self.execution_order = (0..self.passes.len()).collect();
    }
    
    pub fn add_pass<P: Into<Pass<'frame>>, T: Clone + 'static>(&mut self, value: P) -> T {

        match value.into() {
            Pass::Raster(mut pass) => {
                
                if let Some(setup) = pass.setup.take() {

                    let mut builder = PassBuilder {
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
                    pass.render_target = builder.render_target;
                }
                
                let r = pass.data.downcast_ref::<T>().cloned().expect("AAAAA");
                self.passes.push(Pass::Raster(pass));
                return r;
            }
        }
    }
}
