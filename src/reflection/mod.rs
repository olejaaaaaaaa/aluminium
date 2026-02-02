use ash::vk;
use naga::front::spv;
use naga::valid::{Capabilities, ValidationFlags, Validator};
use naga::{
    AddressSpace, Binding, GlobalVariable, Module, ResourceBinding, Scalar, ShaderStage,
    StructMember, Type, TypeInner,
};

use crate::core::{ShaderError, VulkanError, VulkanResult};

pub struct ShaderBinding {
    name: String,
    location: u32,
    size: u32,
}

pub struct ShaderReflection {
    use_instancing: bool,
    workgroup: [u32; 3],
    shader_stage: naga::ShaderStage,
}

impl ShaderReflection {
    fn from(module: &naga::Module) -> VulkanResult<Self> {
        let entry = module.entry_points.last().expect("Not found entry point");

        for arg in entry.function.arguments.iter() {
            if let Some(binding) = &arg.binding {
                match binding {
                    Binding::BuiltIn(builtin) => {
                        match builtin {
                            naga::BuiltIn::InstanceIndex => {

                            },
                            _ => {

                            }
                        }
                    },
                    Binding::Location {
                        location,
                        interpolation,
                        sampling,
                        blend_src,
                        per_primitive,
                    } => {
                        let name = arg.name.as_ref().unwrap();
                        let ty = arg.ty;

                        let ty = module.types[ty].clone();

                        match ty.inner {
                            TypeInner::Scalar(scalar) => {
                                println!(
                                    "location(layout = {:?}) {:?}: {:?}",
                                    location, name, scalar.kind
                                )
                            },
                            TypeInner::Vector { size, scalar } => {
                                println!(
                                    "location(layout = {:?}) {:?}: {:?}",
                                    location, name, scalar.kind
                                )
                            },
                            TypeInner::Image {
                                dim,
                                arrayed,
                                class,
                            } => {
                                println!("location(layout = {:?}) {:?}: {:?}", location, name, dim)
                            },
                            TypeInner::Struct { members, span } => for i in members {},
                            _ => {},
                        }
                    },
                }
            }
        }

        Ok(Self {
            use_instancing: false,
            workgroup: entry.workgroup_size,
            shader_stage: entry.stage,
        })
    }

    pub fn new_from_u8(spirv: &[u8]) -> VulkanResult<Self> {

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

        Self::from(&module)
    }

    pub fn new_from_u32(spirv: &[u32]) -> VulkanResult<Self> {
        let spriv_u8: &[u8] = bytemuck::cast_slice(spirv);
        Self::new_from_u8(spriv_u8)
    }
}
