use super::{PresentPassDesc, Source};

pub enum PassDesc {
    Present(PresentPassDesc),
}

impl PassDesc {
    pub fn sources(&self) -> Vec<&Source> {
        match self {
            PassDesc::Present(pass) => {
                vec![
                    &pass.pipeline_desc.vertex_shader,
                    &pass.pipeline_desc.fragment_shader,
                ]
            },
        }
    }
}
