
use super::Source;
use super::PresentPassDesc;

pub enum PassDesc {
    Present(PresentPassDesc)
}

impl PassDesc {
    pub fn sources(&self) -> Vec<&Source> {
        match self {
            PassDesc::Present(pass) => {
                vec![
                    &pass.vertex_shader,
                    &pass.fragment_shader
                ]
            }
        }
    }
}