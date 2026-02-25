use std::collections::HashMap;

use ash::vk;
use bytemuck::{Pod, Zeroable};
use slotmap::*;

use crate::core::{
    load_spv, AttributeDescriptions, BindingDescriptions, DescriptorSetLayoutBuilder, Device,
    FrameBuffer, GpuBufferBuilder, GraphicsPipeline, GraphicsPipelineBuilder, PipelineLayout,
    PipelineLayoutBuilder, Sampler, ShaderBuilder, Vertex, VulkanResult,
};
use crate::reflection::PipelineShaderReflection;
use crate::render_context::RenderContext;
use crate::render_graph::{RasterPipelineDesc, Source};

mod descriptor_manager;
pub use descriptor_manager::*;

mod mesh;
pub use mesh::*;

mod materials;
pub use materials::*;

mod transforms;
pub use transforms::*;

mod renderable;
pub use renderable::*;

new_key_type! {
    pub struct PipelineLayoutHandle;
    pub struct SamplerHandle;
    pub struct RasterPipelineHandle;
    pub struct FrameBufferHandle;
}

pub struct AssetManager {
    pub(crate) mesh: MeshCollection,
    pub(crate) material: MaterialCollection,
    pub(crate) transform: TransformCollection,
    pub(crate) renderable: RenderableCollection,
}

impl AssetManager {
    pub fn new(device: &Device) -> VulkanResult<Self> {
        Ok(Self {
            mesh: MeshCollection::new(),
            material: MaterialCollection::new(),
            transform: TransformCollection::new(device)?,
            renderable: RenderableCollection::new(),
        })
    }
}

pub struct LowLevelManager {
    cache_raster: HashMap<RasterPipelineDesc, (RasterPipelineHandle, PipelineLayoutHandle)>,
    raster_pipeline: SlotMap<RasterPipelineHandle, GraphicsPipeline>,
    pipeline_layout: SlotMap<PipelineLayoutHandle, PipelineLayout>,
    frame_buffer: SlotMap<FrameBufferHandle, FrameBuffer>,
    sampler: SlotMap<SamplerHandle, Sampler>,
}

struct PipelineLayoutDesc {
    sets: Vec<vk::DescriptorSetLayout>,
    push: vk::PushConstantRange,
}

impl LowLevelManager {
    pub fn new() -> Self {
        Self {
            cache_raster: HashMap::new(),
            raster_pipeline: SlotMap::with_key(),
            pipeline_layout: SlotMap::with_key(),
            frame_buffer: SlotMap::with_key(),
            sampler: SlotMap::with_key(),
        }
    }

    fn shader_reflection(
        device: &Device,
        sources: &[&Source],
    ) -> VulkanResult<PipelineShaderReflection> {
        let mut shaders = vec![];

        for source in sources {
            let shader = match source {
                Source::None => panic!("Shader required!"),
                Source::Path(path_buf) => {
                    let bytecode = load_spv(path_buf);
                    ShaderBuilder::new(device).bytecode(&bytecode).build()?
                },
                Source::SpirvU32(bytecode) => {
                    ShaderBuilder::new(device).bytecode(bytecode).build()?
                },
                Source::SpirvU8(bytes) => ShaderBuilder::new(device)
                    .bytecode(bytemuck::cast_slice(bytes))
                    .build()?,
            };
            shaders.push(shader);
        }

        let reflection = PipelineShaderReflection::from_shaders(shaders)?;

        Ok(reflection)
    }

    pub fn create_raster_pipeline(
        &mut self,
        ctx: &RenderContext,
        desc: &RasterPipelineDesc,
    ) -> VulkanResult<(RasterPipelineHandle, PipelineLayoutHandle)> {
        if let Some((pipeline, layout)) = self.cache_raster.get(desc) {
            return Ok((*pipeline, *layout));
        }

        let sources = [&desc.vertex_shader, &desc.fragment_shader];

        let reflection = Self::shader_reflection(&ctx.device, &sources)?;

        let color_blend = vk::PipelineColorBlendAttachmentState::default()
            .color_write_mask(
                vk::ColorComponentFlags::R
                    | vk::ColorComponentFlags::G
                    | vk::ColorComponentFlags::B
                    | vk::ColorComponentFlags::A,
            )
            .blend_enable(false);

        let binds = Vertex::bind_desc();
        let attr = Vertex::attr_desc();

        let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::default()
            .vertex_binding_descriptions(&binds)
            .vertex_attribute_descriptions(&attr);

        let binds = vec![
            vk::DescriptorSetLayoutBinding::default()
                .binding(0)
                .stage_flags(vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .descriptor_count(1),
            vk::DescriptorSetLayoutBinding::default()
                .binding(1)
                .stage_flags(vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .descriptor_count(1),
            vk::DescriptorSetLayoutBinding::default()
                .binding(2)
                .stage_flags(vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT)
                .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                .descriptor_count(10000),
        ];

        let descriptor_set_layout = DescriptorSetLayoutBuilder::new(&ctx.device)
            .bindings(binds)
            .build()?;

        let layout = PipelineLayoutBuilder::new(&ctx.device)
            .set_layouts(vec![descriptor_set_layout.raw])
            .push_constant(vec![vk::PushConstantRange::default()
                .offset(0)
                .size(128)
                .stage_flags(
                    vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                )])
            .build()?;

        let pipeline = GraphicsPipelineBuilder::new(&ctx.device)
            .vertex_shader(reflection.vertex.as_ref().unwrap().raw)
            .fragment_shader(reflection.fragment.as_ref().unwrap().raw)
            .render_pass(ctx.window.render_pass.raw)
            .pipeline_layout(layout.raw)
            .viewport(vec![vk::Viewport::default()
                .x(0.0)
                .y(0.0)
                .width(ctx.window.resolution.width as f32)
                .height(ctx.window.resolution.height as f32)
                .min_depth(0.0)
                .max_depth(1.0)])
            .scissors(vec![vk::Rect2D::default()
                .offset(vk::Offset2D { x: 0, y: 0 })
                .extent(ctx.window.resolution)])
            .input_assembly(
                vk::PipelineInputAssemblyStateCreateInfo::default()
                    .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
                    .primitive_restart_enable(false),
            )
            .rasterization(
                vk::PipelineRasterizationStateCreateInfo::default()
                    .depth_clamp_enable(false)
                    .rasterizer_discard_enable(false)
                    .polygon_mode(vk::PolygonMode::FILL)
                    .line_width(1.0)
                    .cull_mode(vk::CullModeFlags::NONE)
                    .front_face(vk::FrontFace::CLOCKWISE)
                    .depth_bias_enable(false),
            )
            .multisampling(
                vk::PipelineMultisampleStateCreateInfo::default()
                    .sample_shading_enable(false)
                    .rasterization_samples(vk::SampleCountFlags::TYPE_1),
            )
            .color_blending(
                vk::PipelineColorBlendStateCreateInfo::default()
                    .logic_op_enable(false)
                    .logic_op(vk::LogicOp::COPY)
                    .attachments(&[color_blend]),
            )
            .dynamic_state(vec![vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR])
            .vertex_input_info(vertex_input_info)
            .build()?;

        let pipeline = self.raster_pipeline.insert(pipeline);
        let layout = self.pipeline_layout.insert(layout);

        self.cache_raster.insert(desc.clone(), (pipeline, layout));

        Ok((pipeline, layout))
    }
}

pub struct ResourceManager {
    pub(crate) assets: AssetManager,
    pub(crate) low_level: LowLevelManager,
}

impl ResourceManager {
    pub fn destroy(&mut self, device: &Device) {
        for i in &mut self.assets.mesh.data {
            unsafe {
                if let Some(index) = &mut i.index_buffer {
                    device
                        .allocator
                        .destroy_buffer(index.raw, &mut index.allocation);
                }
                if let Some(instance) = &mut i.instance_buffer {
                    device
                        .allocator
                        .destroy_buffer(instance.raw, &mut instance.allocation);
                }
                device
                    .allocator
                    .destroy_buffer(i.vertex_buffer.raw, &mut i.vertex_buffer.allocation);
            }
        }

        self.assets.transform.destroy(device);

        for (_, pipeline) in self.low_level.raster_pipeline.drain() {
            unsafe { device.destroy_pipeline(pipeline.raw, None) };
        }

        for (_, layout) in self.low_level.pipeline_layout.drain() {
            unsafe {
                device.destroy_pipeline_layout(layout.raw, None);
            }
        }

        for (_, framebuffer) in self.low_level.frame_buffer.drain() {
            unsafe {
                device.destroy_framebuffer(framebuffer.raw, None);
            }
        }
    }

    pub fn new(device: &Device) -> VulkanResult<Self> {
        Ok(Self {
            assets: AssetManager::new(device)?,
            low_level: LowLevelManager::new(),
        })
    }

    pub fn get_renderables(&self) -> Vec<Renderable> {
        self.assets.renderable.get_renderables().clone()
    }

    pub fn get_mesh(&self, handle: MeshHandle) -> &Mesh {
        self.assets.mesh.get_mesh(handle)
    }

    pub fn add_raster_pipeline(&mut self, pipeline: GraphicsPipeline) -> RasterPipelineHandle {
        self.low_level.raster_pipeline.insert(pipeline)
    }

    pub fn add_layout(&mut self, pipeline: PipelineLayout) -> PipelineLayoutHandle {
        self.low_level.pipeline_layout.insert(pipeline)
    }

    pub fn get_layout(&mut self, pipeline: PipelineLayoutHandle) -> Option<&PipelineLayout> {
        self.low_level.pipeline_layout.get(pipeline)
    }

    pub fn get_raster_pipeline(&self, pipeline: RasterPipelineHandle) -> Option<&GraphicsPipeline> {
        self.low_level.raster_pipeline.get(pipeline)
    }

    pub fn get_framebuffer(&self, frame_buffer: FrameBufferHandle) -> Option<&FrameBuffer> {
        self.low_level.frame_buffer.get(frame_buffer)
    }

    pub fn create_static_mesh_immediately<
        T: AttributeDescriptions + BindingDescriptions + Pod + Zeroable,
    >(
        &mut self,
        ctx: &RenderContext,
        mesh: &[T],
        indices: Option<&[u32]>,
    ) -> VulkanResult<MeshHandle> {
        let size = std::mem::size_of_val(mesh) as u64;

        let mut vertex_buffer = GpuBufferBuilder::cpu_only(&ctx.device)
            .size(size)
            .usage(vk::BufferUsageFlags::VERTEX_BUFFER)
            .build()?;

        vertex_buffer.upload_data(&ctx.device, mesh)?;

        let index_buffer = if let Some(indices) = indices {
            let mut index_buffer = GpuBufferBuilder::cpu_only(&ctx.device)
                .size(std::mem::size_of_val(indices) as u64)
                .usage(vk::BufferUsageFlags::INDEX_BUFFER)
                .build()?;

            index_buffer.upload_data(&ctx.device, indices)?;

            Some(index_buffer)
        } else {
            None
        };

        Ok(self.assets.mesh.add_mesh(Mesh {
            instance_offset: 0,
            instance_count: 1,
            vertex_offset: 0,
            instance_buffer: None,
            vertex_buffer,
            indices: indices.map(|x| x.to_vec()),
            index_buffer,
        }))
    }

    pub fn create_renderable(&mut self, renderable: Renderable) -> RenderableHandle {
        self.assets.renderable.add_renderable(renderable)
    }

    pub fn create_material(&mut self, material: Material) -> VulkanResult<MaterialHandle> {
        self.assets.material.add_material(material)
    }

    pub fn create_transform(&mut self, transform: Transform) -> VulkanResult<TransformHandle> {
        self.assets.transform.create_transform(transform)
    }
}
