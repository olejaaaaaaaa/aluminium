use super::{PresentPassDesc, Source};
use crate::render_graph::RasterPassDesc;

pub enum PassDesc {
    Present(PresentPassDesc),
    Raster(RasterPassDesc),
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
            PassDesc::Raster(_pass) => {
                vec![]
            },
        }
    }
}
