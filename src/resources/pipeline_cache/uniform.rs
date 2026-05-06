use ash::vk;

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

pub struct Uniform {
    pub binding: UniformBinding,
    pub ty: UniformType,
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
    pub stage: ShaderStage,
}

pub enum UniformType {
    StorageBuffer,
    UniformBuffer,
    Texture,
    StorageTexture,
}
