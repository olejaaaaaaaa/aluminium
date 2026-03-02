#[derive(Eq, Hash, PartialEq, Clone)]
pub enum ShaderType {
    Mat3x3,
    Mat4x4,
    Float4,
    Float3,
    Float2,
    Float,
    U32,
}

#[derive(Eq, Hash, PartialEq, Clone)]
pub enum ShaderStage {
    Vertex,
    Fragment,
}

#[derive(Eq, Hash, PartialEq, Clone)]
pub struct UniformBinding {
    pub set: u32,
    pub bind: u32,
    pub ty: ShaderType,
    pub stage: ShaderStage,
}
