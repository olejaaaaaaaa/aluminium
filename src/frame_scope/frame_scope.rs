use std::sync::Arc;

use parking_lot::Mutex;
use super::resources::TemporalFrameGraphResources;
use crate::frame_graph::{Pass, PassBuilder};

pub struct FrameScope<'frame> {
    pub resources: TemporalFrameGraphResources,
    pub passes: Vec<Pass<'frame>>,
    execution_order: Vec<usize>,
}

impl<'frame> FrameScope<'frame> {
    pub fn new() -> Self {
        Self {
            passes: vec![],
            resources: TemporalFrameGraphResources::new(),
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

                let mut builder = PassBuilder {
                    resources: &mut self.resources
                };
                
                if let Some(setup) = pass.setup.take() {
                    let data = (setup)(&mut builder);
                    pass.data = data;
                }
                
                let r = pass.data.downcast_ref::<T>().cloned().expect("AAAAA");
                self.passes.push(Pass::Raster(pass));
                return r;
            }
        }
    }
}
