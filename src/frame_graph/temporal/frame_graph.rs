use std::sync::Arc;

use parking_lot::Mutex;

use crate::frame_graph::temporal::TemporalFrameGraphResources;
use crate::frame_graph::{Pass, PassBuilder, PassData};

pub struct TemporalFrameGraph<'frame> {
    pub resources: TemporalFrameGraphResources,
    pub passes: Vec<Pass<'frame>>,
    execution_order: Vec<usize>,
}

impl<'frame> TemporalFrameGraph<'frame> {
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

    pub fn add_pass<P: Into<Pass<'frame>>>(&mut self, value: P) {
        self.passes.push(value.into());
    }
}
