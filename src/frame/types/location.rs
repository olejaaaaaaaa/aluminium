use crate::ShaderStage;

pub struct Location {
    pub stage: ShaderStage,
    pub set: u32,
    pub binding: u32,
}
