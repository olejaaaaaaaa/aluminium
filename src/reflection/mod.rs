use ash::vk;
use naga::front::spv;
use naga::valid::{Capabilities, ValidationFlags, Validator};
use naga::{
    AddressSpace, Binding, GlobalVariable, Module, ResourceBinding, Scalar, ShaderStage,
    StructMember, Type, TypeInner,
};

use crate::core::{ShaderError, VulkanError, VulkanResult};

pub struct PipelineBinding {
    name: String,
    location: u32,
    size: u32,
}

pub struct PipelineReflection {
    use_instancing: bool,
    workgroup: [u32; 3],
    shader_stage: naga::ShaderStage,
}

impl PipelineReflection {
    pub fn new_from_u8(spirv_bytes: &[u8]) -> VulkanResult<PipelineReflection> {
        let options = spv::Options {
            adjust_coordinate_space: false,
            strict_capabilities: false,
            block_ctx_dump_prefix: None,
        };

        println!("BBBB");

        let module = spv::parse_u8_slice(spirv_bytes, &options);

        println!("ERR: {:?}", module.as_ref().err());

        let module = module.map_err(|e| VulkanError::Unknown(vk::Result::from_raw(0)))?;

        println!("AAAA");

        let entry = module.entry_points.last().expect("Not found entry point");

        println!("stage: {:?}", entry.stage);
        println!("workgroup: {:?}", entry.workgroup_size);

        for arg in entry.function.arguments.iter() {
            if let Some(binding) = &arg.binding {
                match binding {
                    Binding::BuiltIn(builtin) => {},
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

        Ok(PipelineReflection {
            use_instancing: false,
            workgroup: entry.workgroup_size,
            shader_stage: entry.stage,
        })
    }

    pub fn new_from_u32(spirv_bytes: &[u32]) -> VulkanResult<PipelineReflection> {
        let options = spv::Options {
            adjust_coordinate_space: false,
            strict_capabilities: false,
            block_ctx_dump_prefix: None,
        };

        let module =
            spv::parse_u8_slice(bytemuck::cast_slice(spirv_bytes), &options).map_err(|e| {
                VulkanError::Shader(ShaderError::ShaderReflectionModuleCreationFailed(format!(
                    "Error: {:?}",
                    e.to_string()
                )))
            })?;

        let entry = module.entry_points.last().expect("Not found entry point");

        println!("stage: {:?}", entry.stage);
        println!("workgroup: {:?}", entry.workgroup_size);

        for arg in entry.function.arguments.iter() {
            if let Some(binding) = &arg.binding {
                match binding {
                    Binding::BuiltIn(builtin) => {},
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

        Ok(PipelineReflection {
            use_instancing: false,
            workgroup: entry.workgroup_size,
            shader_stage: entry.stage,
        })
    }
}
