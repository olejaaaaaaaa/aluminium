use std::sync::Arc;

use ash::vk;
use tracing::debug;

use crate::core::{
    AttributeDescriptions, BindingDescriptions, DescriptorSetLayoutBuilder, Device, GraphicsPipeline, GraphicsPipelineBuilder, PbrVertex, PipelineLayout, PipelineLayoutBuilder, RenderPassBuilder, ShaderBuilder, ShaderModule, Subpass, Vertex, load_spv
};
use crate::resources::pipeline_cache::Source;
use crate::resources::{Create, Res, Resources, ShaderType, Uniform, UniformBinding};
use crate::{ShaderStage, UniformType, VulkanResult};

pub trait Layout {
    fn layout() -> VertexInput;
}

impl Layout for PbrVertex {
    fn layout() -> VertexInput {
        VertexInput::new()
            .attr("position", ShaderType::Float4)
            .attr("normal", ShaderType::Float4)
            .attr("uv", ShaderType::Float2)
            .attr("color", ShaderType::Float4)
            .attr("tangent", ShaderType::Float4)
    }
}

pub struct VertexInput {
    inputs: Vec<ShaderType>,
    #[cfg(feature = "reflection")]
    names: Vec<&'static str>,
}

impl VertexInput {
    pub fn new() -> Self {
        Self {
            inputs: vec![],
            #[cfg(feature = "reflection")]
            names: vec![],
        }
    }

    pub fn attr(mut self, name: &'static str, ty: ShaderType) -> Self {
        self.inputs.push(ty);
        #[cfg(feature = "reflection")]
        self.names.push(name);
        self
    }
}

pub struct RasterPipelineDesc<'a> {
    depth_test: bool,
    dynamic_viewport: bool,
    dynamic_scissors: bool,
    vertex_shader: Option<Source<'a>>,
    fragment_shader: Option<Source<'a>>,
    uniforms: Option<&'a [Uniform]>,
    render_targets: usize,
    vertex_input: VertexInput,
}

impl<'a> Default for RasterPipelineDesc<'a> {
    fn default() -> Self {
        Self {
            depth_test: false,
            dynamic_viewport: false,
            dynamic_scissors: false,
            uniforms: None,
            vertex_shader: None,
            fragment_shader: None,
            render_targets: 1,
            vertex_input: VertexInput::new(),
        }
    }
}

impl<'a> RasterPipelineDesc<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn depth_test(mut self, value: bool) -> Self {
        self.depth_test = value;
        self
    }

    pub fn vertex_shader<Src: Into<Source<'a>>>(mut self, src: Src) -> Self {
        self.vertex_shader = Some(src.into());
        self
    }

    pub fn fragment_shader<Src: Into<Source<'a>>>(mut self, src: Src) -> Self {
        self.fragment_shader = Some(src.into());
        self
    }

    pub fn vertex_input<T: Layout>(mut self) -> Self {
        self.vertex_input = T::layout();
        self
    }

    pub fn uniforms(mut self, uniforms: &'a [Uniform]) -> Self {
        self.uniforms = Some(uniforms);
        self
    }

    pub fn dynamic_viewport(mut self, value: bool) -> Self {
        self.dynamic_viewport = value;
        self
    }

    pub fn dynamic_scissors(mut self, value: bool) -> Self {
        self.dynamic_scissors = value;
        self
    }
}

pub struct RasterPipeline {
    pub layout: Res<PipelineLayout>,
    pub pipeline: GraphicsPipeline,
}

impl Create for RasterPipeline {
    type Desc<'a> = RasterPipelineDesc<'a>;
    fn create(
        ctx: &std::sync::Arc<crate::render_context::RenderContext>,
        resources: &std::sync::Arc<Resources>,
        desc: Self::Desc<'_>,
    ) -> VulkanResult<Res<Self>> {

        let mut offset = 0u32;
        let mut vertex_input_attrs = vec![];

        for (location, ty) in desc.vertex_input.inputs.iter().enumerate() {
            let (format, size) = shader_type_info(ty);

            vertex_input_attrs.push(
                vk::VertexInputAttributeDescription::default()
                    .binding(0)
                    .offset(offset)
                    .location(location as u32)
                    .format(format)
            );

            offset += size;
        }

        let stride = offset;

        let binding = vec![
            vk::VertexInputBindingDescription::default()
                .binding(0)
                .input_rate(vk::VertexInputRate::VERTEX)
                .stride(stride)
        ];

        debug!("Vertex Attrs: {:?}", vertex_input_attrs);

        let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::default()
            .vertex_binding_descriptions(&binding)
            .vertex_attribute_descriptions(&vertex_input_attrs);

        let mut bindings = vec![];

        if let Some(uniforms) = desc.uniforms {
            for i in uniforms {
                match i.ty {
                    UniformType::StorageBuffer => {

                        let flags = match i.binding.stage {
                            ShaderStage::Vertex => {
                                vk::ShaderStageFlags::VERTEX
                            },
                            ShaderStage::Fragment => {
                                vk::ShaderStageFlags::FRAGMENT
                            }
                        };

                        bindings.push(
                            vk::DescriptorSetLayoutBinding::default()
                                .binding(i.binding.binding)
                                .descriptor_count(1)
                                .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                                .stage_flags(flags)
                        );
                    },
                    _ => unimplemented!()
                }
            }
        }

        debug!("Vertex Bindings: {:?}", bindings);

        let mut set_layouts = vec![resources.bindless.set_layout.raw];

        if !bindings.is_empty() {
            let set_layout = DescriptorSetLayoutBuilder::new(&ctx.device)
                .bindings(bindings)
                .build()?;

            set_layouts.push(set_layout.raw);
            std::mem::forget(set_layout);
        }

        debug!("Set Layouts: {:?}", set_layouts);

        let layout = PipelineLayoutBuilder::new(&ctx.device)
            .set_layouts(set_layouts)
            .push_constant(vec![vk::PushConstantRange::default()
                .offset(0)
                .size(128)
                .stage_flags(
                    vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                )])
            .build()?;

        let vertex_src = desc.vertex_shader.expect("Missing Vertex Shader");
        let vertex_shader = create_shader(&ctx.device, vertex_src)?;

        let fragment_src = desc.fragment_shader.expect("Missing Fragment Shader");
        let fragment_shader = create_shader(&ctx.device, fragment_src)?;

        let mut dynamic_states = Vec::with_capacity(2);

        if desc.dynamic_viewport {
            dynamic_states.push(vk::DynamicState::VIEWPORT);
        }

        if desc.dynamic_scissors {
            dynamic_states.push(vk::DynamicState::SCISSOR);
        }

        debug!("Dynamic states: {:?}", dynamic_states);

        let mut attachments = vec![];

        for _ in 0..desc.render_targets {
            let color_attachment = vk::AttachmentDescription::default()
                .initial_layout(vk::ImageLayout::UNDEFINED)
                .final_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                .format(vk::Format::R8G8B8A8_SRGB)
                .flags(vk::AttachmentDescriptionFlags::empty())
                .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
                .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
                .samples(vk::SampleCountFlags::TYPE_1)
                .store_op(vk::AttachmentStoreOp::DONT_CARE)
                .load_op(vk::AttachmentLoadOp::DONT_CARE);

            attachments.push(color_attachment);
        }

        if desc.depth_test {
            let depth_attachment = vk::AttachmentDescription::default()
                .initial_layout(vk::ImageLayout::UNDEFINED)
                .final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
                .format(vk::Format::D32_SFLOAT)
                .flags(vk::AttachmentDescriptionFlags::empty())
                .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
                .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
                .samples(vk::SampleCountFlags::TYPE_1)
                .store_op(vk::AttachmentStoreOp::DONT_CARE)
                .load_op(vk::AttachmentLoadOp::DONT_CARE);

            attachments.push(
                depth_attachment
            );
        }

        let dependency = vec![vk::SubpassDependency {
            src_subpass: vk::SUBPASS_EXTERNAL,
            dst_subpass: 0,
            src_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
                | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
            dst_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
                | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
            src_access_mask: vk::AccessFlags::empty(),
            dst_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_WRITE
                | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
            dependency_flags: vk::DependencyFlags::BY_REGION,
        }];

        let mut subpass = Subpass::new(vk::PipelineBindPoint::GRAPHICS);

        for (index, attach) in attachments.iter().enumerate() {
            match attach.final_layout {
                vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL => {
                    subpass = subpass.add_color_attachment_ref(
                        vk::AttachmentReference::default()
                        .attachment(index as u32)
                        .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                    );
                },
                vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL => {
                    subpass = subpass.add_depth_attachment_ref(
                        vk::AttachmentReference::default()
                            .attachment(index as u32)
                            .layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
                    );
                },
                x => unimplemented!("layout: {:?}", x)
            }
        }

        let subpasses = vec![subpass];

        let render_pass = RenderPassBuilder::new(&ctx.device)
            .attachments(attachments)
            .subpasses(subpasses)
            .dependencies(dependency)
            .build()?;

        let resolution = ctx.resolution();
        let color_blend = vk::PipelineColorBlendAttachmentState::default()
            .color_write_mask(
                vk::ColorComponentFlags::R
                    | vk::ColorComponentFlags::G
                    | vk::ColorComponentFlags::B
                    | vk::ColorComponentFlags::A,
            )
            .blend_enable(false);

        let pipeline = GraphicsPipelineBuilder::new(&ctx.device)
            .vertex_shader(vertex_shader.raw)
            .fragment_shader(fragment_shader.raw)
            .render_pass(render_pass.raw)
            .pipeline_layout(layout.raw)
            .viewport(vec![vk::Viewport::default()
                .x(0.0)
                .y(0.0)
                .width(resolution.width as f32)
                .height(resolution.height as f32)
                .min_depth(0.0)
                .max_depth(1.0)])
            .scissors(vec![vk::Rect2D::default()
                .offset(vk::Offset2D { x: 0, y: 0 })
                .extent(resolution)])
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
            .dynamic_state(dynamic_states)
            .vertex_input_info(vertex_input_info)
            .build()?;

        std::mem::forget(render_pass);

        let cache = resources.pipeline_cache.write();

        let layout = cache.pipeline_layout.insert(layout);

        let layout = resources.make_handle(ctx, layout);
        let pipeline = cache
            .raster_pipelines
            .insert(RasterPipeline { layout, pipeline });

        Ok(resources.make_handle(ctx, pipeline))
    }
}

fn shader_type_info(ty: &ShaderType) -> (vk::Format, u32) {
    match ty {
        ShaderType::Float  => (vk::Format::R32_SFLOAT,          4),
        ShaderType::Float2 => (vk::Format::R32G32_SFLOAT,       8),
        ShaderType::Float3 => (vk::Format::R32G32B32_SFLOAT,    12),
        ShaderType::Float4 => (vk::Format::R32G32B32A32_SFLOAT, 16),
        _ => unimplemented!()
    }
}

fn create_shader(device: &Device, src: Source<'_>) -> VulkanResult<ShaderModule> {
    match src {
        Source::Path(path) => {
            let spv = load_spv(path).expect("Path not found or SPIR-V bytecode not valid");
            Ok(ShaderBuilder::new(device)
                .bytecode(&spv)
                .build()?)
        },
        Source::SpirvU32(bytecode) => {
            Ok(ShaderBuilder::new(device)
                .bytecode(&bytecode)
                .build()?)
        },
        Source::SpirvU8(bytcode) => {
            let bytecode = bytemuck::cast_slice(bytcode);
            Ok(ShaderBuilder::new(device)
                .bytecode(bytecode)
                .build()?)
        }
    }
}