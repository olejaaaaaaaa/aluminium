use std::collections::HashMap;

use ash::vk;
use naga::front::spv;

use crate::core::{ShaderError, ShaderModule, VulkanError, VulkanResult};

pub struct ShaderReflection {
    pub shader: vk::ShaderModule,
    pub shader_stage: naga::ShaderStage,

    /// Vertex shader inputs
    #[allow(dead_code)]
    pub vertex_inputs: Vec<VertexInputAttribute>,

    /// Fragment shader outputs
    #[allow(dead_code)]
    pub fragment_outputs: Vec<FragmentOutput>,

    /// Descriptor sets (uniform buffers, textures, samplers)
    #[allow(dead_code)]
    pub descriptor_sets: HashMap<u32, Vec<DescriptorBinding>>,

    /// Push constants
    #[allow(dead_code)]
    pub push_constants: Option<PushConstantInfo>,

    /// Flags
    #[allow(dead_code)]
    pub use_instancing: bool,
    #[allow(dead_code)]
    pub use_bindless: bool,
    #[allow(dead_code)]
    pub workgroup: [u32; 3],
}

#[derive(Debug, Clone)]
pub struct VertexInputAttribute {
    pub location: u32,
    #[allow(dead_code)]
    pub name: String,
    #[allow(dead_code)]
    pub format: vk::Format,
    #[allow(dead_code)]
    pub offset: u32,
}

#[derive(Debug, Clone)]
pub struct FragmentOutput {
    pub location: u32,
    #[allow(dead_code)]
    pub name: String,
    #[allow(dead_code)]
    pub format: vk::Format,
}

#[derive(Debug, Clone)]
pub struct DescriptorBinding {
    pub binding: u32,
    #[allow(dead_code)]
    pub descriptor_type: vk::DescriptorType,
    #[allow(dead_code)]
    pub stage_flags: vk::ShaderStageFlags,
    #[allow(dead_code)]
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct PushConstantInfo {
    #[allow(dead_code)]
    pub size: u32,
    #[allow(dead_code)]
    pub stage_flags: vk::ShaderStageFlags,
}

impl ShaderReflection {
    fn from(shader: vk::ShaderModule, module: &naga::Module) -> VulkanResult<Self> {
        let entry = module.entry_points.first().unwrap();

        let shader_stage = entry.stage;
        let workgroup = entry.workgroup_size;

        // 1. Извлекаем INPUTS (vertex attributes или fragment inputs)
        let vertex_inputs = Self::extract_inputs(module, entry)?;

        // 2. Извлекаем OUTPUTS (fragment outputs)
        let fragment_outputs = Self::extract_outputs(module, entry)?;

        // 3. Извлекаем DESCRIPTORS (uniforms, textures, storage buffers)
        let descriptor_sets = Self::extract_descriptors(module)?;

        // 4. Извлекаем PUSH CONSTANTS
        let push_constants = Self::extract_push_constants(module)?;

        // 5. Проверяем спецальные флаги
        let use_instancing = Self::check_instancing(entry);
        let use_bindless = Self::check_bindless(module);

        Ok(Self {
            shader,
            shader_stage,
            vertex_inputs,
            fragment_outputs,
            descriptor_sets,
            push_constants,
            use_instancing,
            use_bindless,
            workgroup,
        })
    }

    // ============ INPUTS (location = X) ============
    fn extract_inputs(
        module: &naga::Module,
        entry: &naga::EntryPoint,
    ) -> VulkanResult<Vec<VertexInputAttribute>> {
        let mut inputs = Vec::new();
        let mut current_offset = 0u32;

        for arg in &entry.function.arguments {
            if let Some(naga::Binding::Location { location, .. }) = &arg.binding {
                let name = arg
                    .name
                    .clone()
                    .unwrap_or_else(|| format!("input_{}", location));
                let ty = &module.types[arg.ty];

                let format = Self::naga_type_to_vk_format(&ty.inner)?;
                let size = Self::calculate_type_size(&ty.inner);

                inputs.push(VertexInputAttribute {
                    location: *location,
                    name,
                    format,
                    offset: current_offset,
                });

                current_offset += size;
            }
        }

        inputs.sort_by_key(|a| a.location);
        Ok(inputs)
    }

    // ============ OUTPUTS (fragment shader) ============
    fn extract_outputs(
        module: &naga::Module,
        entry: &naga::EntryPoint,
    ) -> VulkanResult<Vec<FragmentOutput>> {
        let mut outputs = Vec::new();

        // Проверяем return type функции
        if let Some(result) = &entry.function.result {
            if let Some(naga::Binding::Location { location, .. }) = &result.binding {
                let ty = &module.types[result.ty];
                let format = Self::naga_type_to_vk_format(&ty.inner)?;

                outputs.push(FragmentOutput {
                    location: *location,
                    name: "output".to_string(),
                    format,
                });
            }
        }

        outputs.sort_by_key(|o| o.location);
        Ok(outputs)
    }

    // ============ DESCRIPTORS (set = X, binding = Y) ============
    fn extract_descriptors(
        module: &naga::Module,
    ) -> VulkanResult<HashMap<u32, Vec<DescriptorBinding>>> {
        let mut sets: HashMap<u32, Vec<DescriptorBinding>> = HashMap::new();

        for (_handle, global) in module.global_variables.iter() {
            if let Some(naga::ResourceBinding { group, binding }) = global.binding {
                let name = global
                    .name
                    .clone()
                    .unwrap_or_else(|| format!("binding_{}", binding));
                let ty = &module.types[global.ty];

                let descriptor_type = match global.space {
                    naga::AddressSpace::Uniform => vk::DescriptorType::UNIFORM_BUFFER,
                    naga::AddressSpace::Storage { .. } => vk::DescriptorType::STORAGE_BUFFER,
                    naga::AddressSpace::Handle => {
                        // Определяем тип по TypeInner
                        match &ty.inner {
                            naga::TypeInner::Image { .. } => vk::DescriptorType::SAMPLED_IMAGE,
                            naga::TypeInner::Sampler { .. } => vk::DescriptorType::SAMPLER,
                            _ => continue,
                        }
                    },
                    _ => continue,
                };

                let entry = sets.entry(group).or_insert_with(Vec::new);
                entry.push(DescriptorBinding {
                    binding,
                    descriptor_type,
                    stage_flags: vk::ShaderStageFlags::ALL, // Уточнить при merge
                    name,
                });
            }
        }

        // Сортируем по binding
        for bindings in sets.values_mut() {
            bindings.sort_by_key(|b| b.binding);
        }

        Ok(sets)
    }

    // ============ PUSH CONSTANTS ============
    fn extract_push_constants(_module: &naga::Module) -> VulkanResult<Option<PushConstantInfo>> {
        // for (_, global) in module.global_variables.iter() {
        //     if global.space == naga::AddressSpace: {
        //         let ty = &module.types[global.ty];
        //         let size = Self::calculate_type_size(&ty.inner);

        //         return Ok(Some(PushConstantInfo {
        //             size,
        //             stage_flags: vk::ShaderStageFlags::ALL,
        //         }));
        //     }
        // }

        Ok(None)
    }

    // ============ HELPER FUNCTIONS ============

    fn check_instancing(entry: &naga::EntryPoint) -> bool {
        entry.function.arguments.iter().any(|arg| {
            matches!(
                arg.binding,
                Some(naga::Binding::BuiltIn(naga::BuiltIn::InstanceIndex))
            )
        })
    }

    fn check_bindless(module: &naga::Module) -> bool {
        // Проверяем есть ли unbounded arrays
        module.global_variables.iter().any(|(_, global)| {
            let ty = &module.types[global.ty];
            matches!(ty.inner, naga::TypeInner::BindingArray { .. })
        })
    }

    fn naga_type_to_vk_format(ty: &naga::TypeInner) -> VulkanResult<vk::Format> {
        match ty {
            naga::TypeInner::Scalar(scalar) => match scalar.kind {
                naga::ScalarKind::Float => Ok(vk::Format::R32_SFLOAT),
                naga::ScalarKind::Sint => Ok(vk::Format::R32_SINT),
                naga::ScalarKind::Uint => Ok(vk::Format::R32_UINT),
                _ => Err(VulkanError::Shader(ShaderError::UnsupportedType)),
            },
            naga::TypeInner::Vector { size, scalar } => match (size, scalar.kind, scalar.width) {
                (naga::VectorSize::Bi, naga::ScalarKind::Float, 4) => Ok(vk::Format::R32G32_SFLOAT),
                (naga::VectorSize::Tri, naga::ScalarKind::Float, 4) => {
                    Ok(vk::Format::R32G32B32_SFLOAT)
                },
                (naga::VectorSize::Quad, naga::ScalarKind::Float, 4) => {
                    Ok(vk::Format::R32G32B32A32_SFLOAT)
                },
                (naga::VectorSize::Bi, naga::ScalarKind::Sint, 4) => Ok(vk::Format::R32G32_SINT),
                (naga::VectorSize::Tri, naga::ScalarKind::Sint, 4) => {
                    Ok(vk::Format::R32G32B32_SINT)
                },
                (naga::VectorSize::Quad, naga::ScalarKind::Sint, 4) => {
                    Ok(vk::Format::R32G32B32A32_SINT)
                },
                _ => Err(VulkanError::Shader(ShaderError::UnsupportedType)),
            },
            naga::TypeInner::Matrix {
                columns,
                rows,
                scalar,
            } => {
                // Матрицы обрабатываются как несколько vec атрибутов
                match (columns, rows, scalar.kind) {
                    (naga::VectorSize::Quad, naga::VectorSize::Quad, naga::ScalarKind::Float) => {
                        Ok(vk::Format::R32G32B32A32_SFLOAT) // mat4 = 4 vec4
                    },
                    _ => Err(VulkanError::Shader(ShaderError::UnsupportedType)),
                }
            },
            _ => Err(VulkanError::Shader(ShaderError::UnsupportedType)),
        }
    }

    fn calculate_type_size(ty: &naga::TypeInner) -> u32 {
        match ty {
            naga::TypeInner::Scalar(scalar) => scalar.width as u32,
            naga::TypeInner::Vector { size, scalar } => {
                let count = match size {
                    naga::VectorSize::Bi => 2,
                    naga::VectorSize::Tri => 3,
                    naga::VectorSize::Quad => 4,
                };
                count * scalar.width as u32
            },
            naga::TypeInner::Matrix {
                columns,
                rows,
                scalar,
            } => {
                let col_count = match columns {
                    naga::VectorSize::Bi => 2,
                    naga::VectorSize::Tri => 3,
                    naga::VectorSize::Quad => 4,
                };
                let row_count = match rows {
                    naga::VectorSize::Bi => 2,
                    naga::VectorSize::Tri => 3,
                    naga::VectorSize::Quad => 4,
                };
                col_count * row_count * scalar.width as u32
            },
            naga::TypeInner::Struct { members, .. } => {
                members
                    .iter()
                    .map(|_m| {
                        // Рекурсивно для вложенных структур
                        // Упрощённо - нужен правильный alignment
                        16 // placeholder
                    })
                    .sum()
            },
            _ => 0,
        }
    }

    pub fn new_from_u8(shader: vk::ShaderModule, spirv: &[u8]) -> VulkanResult<Self> {
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

        Self::from(shader, &module)
    }

    pub fn new_from_shader(shader: &ShaderModule) -> VulkanResult<Self> {
        Self::new_from_u32(shader.raw, shader.spirv_bytes.as_ref().unwrap())
    }

    pub fn new_from_u32(shader: vk::ShaderModule, spirv: &[u32]) -> VulkanResult<Self> {
        let spriv_u8: &[u8] = bytemuck::cast_slice(spirv);
        Self::new_from_u8(shader, spriv_u8)
    }
}
