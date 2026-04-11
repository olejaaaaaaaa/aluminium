use crate::{Handle, Res, frame_graph::Pass, resources::Destroy};

pub struct TemporalFrameGraph<'frame> {
    pub passes: Vec<Pass<'frame>>,
    execution_order: Vec<usize>,
}

impl<'frame> TemporalFrameGraph<'frame> {

    pub fn new() -> Self {
        Self {
            passes: vec![],
            execution_order: vec![]
        }
    }

    fn topological_sort(&mut self) {
        profiling::scope!("FrameGraph::topological_sort");
        self.execution_order = (0..self.passes.len()).collect();
    }

    pub fn create<T>(value: T) -> Handle<T> {
        todo!()
    }

    pub fn add_pass<P: Into<Pass<'frame>>>(&mut self, pass: P) {
        self.passes.push(pass.into());
    }

    pub fn import<T: Destroy>(res: &Res<T>) -> Handle<T> {
        todo!()
    }
}