#![allow(missing_docs)]

pub struct ComputePipelineDesc {
    #[allow(dead_code)]
    compute_shader: String,
}

pub struct ComputePass {
    #[allow(dead_code)]
    name: String,
    #[allow(dead_code)]
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
