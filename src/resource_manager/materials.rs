use std::collections::HashMap;

use crate::core::VulkanResult;
use crate::render_graph::TextureHandle;

#[derive(Clone, Copy)]
pub struct MaterialHandle(pub usize);

pub struct MaterialCollection {
    data: Vec<Material>,
}

impl MaterialCollection {
    pub fn new() -> Self {
        Self { data: vec![] }
    }

    pub fn add_material(&mut self, material: Material) -> VulkanResult<MaterialHandle> {
        let index = self.data.len();
        self.data.push(material);
        Ok(MaterialHandle(index))
    }

    #[allow(dead_code)]
    pub fn get_mut_material(&mut self, handle: MaterialHandle) -> &mut Material {
        &mut self.data[handle.0]
    }
}

#[derive(Clone)]
pub enum UniformValue {
    Bool(bool),
    Float(f32),
    Uint(u32),
    Float2([f32; 2]),
    Float3([f32; 3]),
    Texture(TextureHandle),
}

impl std::ops::Add<f32> for UniformValue {
    type Output = Option<UniformValue>;

    fn add(self, other: f32) -> Self::Output {
        match self {
            UniformValue::Float(a) => Some(UniformValue::Float(a + other)),
            _ => None,
        }
    }
}

impl From<f32> for UniformValue {
    fn from(val: f32) -> Self {
        UniformValue::Float(val)
    }
}

impl From<[f32; 3]> for UniformValue {
    fn from(value: [f32; 3]) -> Self {
        UniformValue::Float3(value)
    }
}

/// Material
#[derive(Clone)]
pub struct Material {
    pub(crate) set: u32,
    pub(crate) uniforms: HashMap<String, UniformValue>,
}

impl Material {
    /// Create new Material
    #[allow(dead_code)]
    pub fn new(set: u32) -> Self {
        Material {
            set,
            uniforms: HashMap::new(),
        }
    }

    /// Setup Uniform Value
    #[allow(dead_code)]
    pub fn set_value<S: Into<String>, T: Into<UniformValue>>(mut self, name: S, value: T) -> Self {
        self.uniforms.insert(name.into(), value.into());
        self
    }

    /// Get mut Uniform Value
    #[allow(dead_code)]
    pub fn get_mut<S: Into<String>, T: Into<UniformValue>>(
        &mut self,
        name: S,
    ) -> Option<&mut UniformValue> {
        self.uniforms.get_mut(&name.into())
    }
}
