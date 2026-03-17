use ash::vk;

pub enum Slot {
    Bindless { index: u32 },
    Uniform { set: u32, binding: u32 },
}

#[derive(Eq, Hash, PartialEq, Clone)]
pub enum ShaderType {
    Custom(vk::Format),
    Texture2D,
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
    pub binding: u32,
    pub ty: ShaderType,
}
