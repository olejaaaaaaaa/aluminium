use std::collections::HashMap;

use ash::vk;
use naga::front::spv;
use naga::{Binding, BuiltIn, TypeInner};

use crate::core::{ShaderError, ShaderModule, VulkanError, VulkanResult};

pub struct ShaderReflection {
    pub raw: vk::ShaderModule,
    pub stage: naga::ShaderStage,
    pub inputs: Vec<vk::VertexInputAttributeDescription>,
    pub descriptor_sets: HashMap<u32, Vec<DescriptorBinding>>,
    pub workgroup: [u32; 3],
}

impl ShaderReflection {
    pub fn from(shader: vk::ShaderModule, module: &naga::Module) -> VulkanResult<Self> {
        let mut inputs = vec![];
        let mut offset = 0;

        let entry = module.entry_points.first().expect("Not found entry point!");

        for i in &entry.function.arguments {
            let name = i.name.clone();

            println!("Name: {:?}", name);

            let binding = i.binding.clone();
            let ty = module.types[i.ty].clone();

            if let Some(bind) = binding {
                match bind {
                    Binding::BuiltIn(builtin) => match builtin {
                        BuiltIn::BaseInstance => {},
                        BuiltIn::BaseVertex => {},
                        BuiltIn::VertexIndex => {
                            println!("Vertex Index");
                        },
                        BuiltIn::InstanceIndex => {},
                        _ => {},
                    },
                    Binding::Location {
                        location,
                        interpolation: _,
                        sampling: _,
                        blend_src: _,
                        per_primitive: _,
                    } => match ty.inner {
                        TypeInner::Scalar(x) => {
                            inputs.push(vk::VertexInputAttributeDescription {
                                location,
                                binding: 0,
                                format: vk::Format::R8G8B8A8_SRGB,
                                offset,
                            });
                            offset += x.width as u32;
                        },
                        TypeInner::Vector { size, scalar } => {
                            inputs.push(vk::VertexInputAttributeDescription {
                                location,
                                binding: 0,
                                format: vk::Format::R8G8B8A8_SRGB,
                                offset,
                            });

                            match size {
                                naga::VectorSize::Bi => {
                                    offset += scalar.width as u32 * 2;
                                },
                                naga::VectorSize::Quad => {
                                    offset += scalar.width as u32 * 4;
                                },
                                naga::VectorSize::Tri => {
                                    offset += scalar.width as u32 * 3;
                                },
                            }
                        },
                        TypeInner::Matrix {
                            columns: _,
                            rows: _,
                            scalar: _,
                        } => {},
                        _ => {},
                    },
                }
            }
        }

        println!("Inputs: {:?}", inputs);

        let stage = entry.stage;
        println!("Stage: {:?}", stage);

        let workgroup = entry.workgroup_size;
        println!("Workgroup: {:?}", workgroup);

        Ok(Self {
            raw: shader,
            stage,
            inputs,
            descriptor_sets: HashMap::new(),
            workgroup,
        })
    }
}

pub struct PipelineShaderReflection {
    pub vertex: Option<ShaderReflection>,
    pub fragment: Option<ShaderReflection>,
    pub compute: Option<ShaderReflection>,
}

#[derive(Debug, Clone)]
pub struct DescriptorBinding {
    pub binding: u32,
    pub descriptor_type: vk::DescriptorType,
    pub stage_flags: vk::ShaderStageFlags,
    pub name: String,
}

impl PipelineShaderReflection {
    pub fn new_from_u8(shader: vk::ShaderModule, spirv: &[u8]) -> VulkanResult<ShaderReflection> {
        let options = spv::Options {
            adjust_coordinate_space: false,
            strict_capabilities: false,
            block_ctx_dump_prefix: None,
        };

        let module = spv::parse_u8_slice(spirv, &options).map_err(|e| {
            VulkanError::Shader(ShaderError::ShaderReflectionModuleCreationFailed(format!(
                "Error: {:?}",
                e.to_string()
            )))
        })?;

        ShaderReflection::from(shader, &module)
    }

    pub fn from_shaders(shaders: Vec<ShaderModule>) -> VulkanResult<Self> {
        let mut pipeline_reflection = PipelineShaderReflection {
            vertex: None,
            fragment: None,
            compute: None,
        };

        for i in shaders {
            let reflection = Self::new_from_u8(i.raw, &i.bytes)?;

            match reflection.stage {
                naga::ShaderStage::Vertex => pipeline_reflection.vertex = Some(reflection),
                naga::ShaderStage::Compute => pipeline_reflection.compute = Some(reflection),
                naga::ShaderStage::Fragment => pipeline_reflection.fragment = Some(reflection),
                _ => panic!("Not supported this type of shader!"),
            }
        }

        Ok(pipeline_reflection)
    }
}
