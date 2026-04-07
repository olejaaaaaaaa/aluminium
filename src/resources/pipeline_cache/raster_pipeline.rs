use std::sync::Arc;

use ash::vk;

use crate::core::{
    AttributeDescriptions, BindingDescriptions, GraphicsPipeline, GraphicsPipelineBuilder, PbrVertex, PipelineLayout, PipelineLayoutBuilder, ShaderBuilder, Vertex, load_spv
};
use crate::resources::pipeline_cache::Source;
use crate::resources::{Create, Destroy, Res, Resources, ShaderType};
use crate::VulkanResult;

pub struct VertexInput {
    inputs: Vec<ShaderType>,
}

impl VertexInput {
    pub fn new() -> Self {
        Self { inputs: vec![] }
    }

    pub fn with(mut self, ty: ShaderType) -> Self {
        self.inputs.push(ty);
        self
    }
}

pub struct RasterPipelineDesc<'a> {
    use_cache: bool,
    dynamic_viewport: bool,
    dynamic_scissors: bool,
    vertex_shader: Option<Source<'a>>,
    fragment_shader: Option<Source<'a>>,
    multiple_render_target: Option<usize>,
    vertex_input: Option<VertexInput>,
}

impl<'a> Default for RasterPipelineDesc<'a> {
    fn default() -> Self {
        Self {
            use_cache: false,
            dynamic_viewport: false,
            dynamic_scissors: false,
            vertex_shader: None,
            fragment_shader: None,
            multiple_render_target: None,
            vertex_input: None,
        }
    }
}

impl<'a> RasterPipelineDesc<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn render_target(mut self, count: usize) -> Self {
        self.multiple_render_target = Some(count);
        self
    }

    pub fn use_cache(mut self, value: bool) -> Self {
        self.use_cache = value;
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

    pub fn vertex_input(mut self, input: VertexInput) -> Self {
        self.vertex_input = Some(input);
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

impl Destroy for RasterPipeline {
    fn destroy(_handle: &Res<Self>, _ctx: std::sync::Weak<crate::render_context::RenderContext>, _resources: std::sync::Weak<Resources>) {}
}

impl Create for RasterPipeline {
    type Desc<'a> = RasterPipelineDesc<'a>;
    fn create(
        ctx: &std::sync::Arc<crate::render_context::RenderContext>,
        resources: &std::sync::Arc<Resources>,
        desc: Self::Desc<'_>,
    ) -> VulkanResult<Res<Self>> {
        let binding = PbrVertex::bind_desc();
        let attrs = PbrVertex::attr_desc();

        let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::default()
            .vertex_binding_descriptions(&binding)
            .vertex_attribute_descriptions(&attrs);

        let layout = PipelineLayoutBuilder::new(&ctx.device)
            .set_layouts(vec![])
            .push_constant(vec![vk::PushConstantRange::default()
                .offset(0)
                .size(128)
                .stage_flags(vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT)])
            .build()?;

        let color_blend = vk::PipelineColorBlendAttachmentState::default()
            .color_write_mask(vk::ColorComponentFlags::R | vk::ColorComponentFlags::G | vk::ColorComponentFlags::B | vk::ColorComponentFlags::A)
            .blend_enable(false);

        let vertex_shader = desc.vertex_shader.unwrap();
        let vertex = match vertex_shader {
            Source::Path(path) => {
                let spv = load_spv(path);
                ShaderBuilder::new(&ctx.device)
                    .bytecode(&spv)
                    .build()
                    .unwrap()
            },
            _ => {
                panic!("AAAA");
            },
        };

        let fragment_shader = desc.fragment_shader.unwrap();
        let fragment = match fragment_shader {
            Source::Path(path) => {
                let spv = load_spv(path);
                ShaderBuilder::new(&ctx.device)
                    .bytecode(&spv)
                    .build()
                    .unwrap()
            },
            _ => {
                panic!("AAAA");
            },
        };

        let resolution = ctx.resolution();

        let pipeline = GraphicsPipelineBuilder::new(&ctx.device)
            .vertex_shader(vertex.raw)
            .fragment_shader(fragment.raw)
            .render_pass(ctx.window.read().render_pass.raw)
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
            .dynamic_state(vec![vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR])
            .vertex_input_info(vertex_input_info)
            .build()?;

        let mut cache = resources.pipeline_cache.write();
        let layout = cache.pipeline_layout.insert(
            Arc::downgrade(ctx), Arc::downgrade(resources),
            layout
        );

        let handle =
            cache.raster_pipelines
                .insert(Arc::downgrade(ctx), Arc::downgrade(resources), RasterPipeline { 
                    pipeline,
                    layout
                });

        Ok(handle)
    }
}
