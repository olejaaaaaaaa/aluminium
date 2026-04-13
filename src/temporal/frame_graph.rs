use crate::frame_graph::{IntoPass, Pass};
use crate::resources::Destroy;
use crate::{Handle, Res};

pub struct TemporalFrameGraph<'frame> {
    pub passes: Vec<Pass<'frame>>,
    execution_order: Vec<usize>,
}

impl<'frame> TemporalFrameGraph<'frame> {
    pub fn new() -> Self {
        Self {
            passes: vec![],
            execution_order: vec![],
        }
    }

    fn topological_sort(&mut self) {
        profiling::scope!("FrameGraph::topological_sort");
        self.execution_order = (0..self.passes.len()).collect();
    }

    pub fn create<T>(value: T) -> Handle<T> {
        todo!()
    }

    pub fn add_pass<T: Copy, P: IntoPass<'frame, T>>(&mut self, pass: P) -> T
    {
        let (pass, data) = pass.into_pass();
        self.passes.push(pass);
        data
    }

    pub fn import<T: Destroy>(res: &Res<T>) -> Handle<T> {
        todo!()
    }
}
