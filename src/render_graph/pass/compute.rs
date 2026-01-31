pub struct ComputePipelineDesc {
    compute_shader: String,
}

pub struct ComputePass {
    name: String,
    desc: ComputePipelineDesc,
}

impl ComputePass {
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            desc: ComputePipelineDesc {
                compute_shader: String::new(),
            },
        }
    }

    pub fn dispatch(&self) {}
}
