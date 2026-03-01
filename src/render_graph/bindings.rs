

#[derive(Eq, Hash, PartialEq, Clone)]
pub enum ShaderType {
    Matrix3x3,
    Matrix4x4,
    Vec4,
    Vec3,
    Float,
    U32
}

#[derive(Eq, Hash, PartialEq, Clone)]
pub enum ShaderStage {
    Vertex,
    Fragment
}

#[derive(Eq, Hash, PartialEq, Clone)]
pub struct UniformBinding {
    pub set: u32,
    pub bind: u32,
    pub ty: ShaderType,
    pub stage: ShaderStage
}