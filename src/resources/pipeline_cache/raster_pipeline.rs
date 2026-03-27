use ash::vk;

use crate::core::{GraphicsPipelineBuilder, PipelineLayoutBuilder};
use crate::resources::pipeline_cache::Source;
use crate::resources::{Create, Destroy, Res, Resources, ShaderType};
use crate::{VulkanError, VulkanResult};

pub struct RasterPipelineDesc {
    use_cache: bool,
    dynamic_viewport: bool,
    dynamic_scissors: bool,
    vertex_shader: Source,
    fragment_shader: Source,
    multiple_render_target: Option<usize>,
    vertex_attributes: Vec<ShaderType>,
}

impl Default for RasterPipelineDesc {
    fn default() -> Self {
        Self {
            use_cache: false,
            dynamic_viewport: false,
            dynamic_scissors: false,
            vertex_shader: Source::None,
            fragment_shader: Source::None,
            multiple_render_target: None,
            vertex_attributes: vec![],
        }
    }
}

impl RasterPipelineDesc {
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

    pub fn vertex_shader<Src: Into<Source>>(mut self, src: Src) -> Self {
        self.vertex_shader = src.into();
        self
    }

    pub fn vertex_attribute(mut self, attr: ShaderType) -> Self {
        self.vertex_attributes.push(attr);
        self
    }

    pub fn fragment_shader<Src: Into<Source>>(mut self, src: Src) -> Self {
        self.fragment_shader = src.into();
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

pub struct RasterPipeline {}

impl Destroy for RasterPipeline {
    fn destroy(handle: &Res<Self>, ctx: std::sync::Weak<crate::render_context::RenderContext>, resources: std::sync::Weak<Resources>) {
        
    }
}

impl Create for RasterPipeline {
    type Desc<'a> = RasterPipelineDesc;
    fn create(ctx: &std::sync::Arc<crate::render_context::RenderContext>, resources: &std::sync::Arc<Resources>, desc: Self::Desc<'_>) -> VulkanResult<Res<Self>> {
        Err(VulkanError::Unknown(vk::Result::from_raw(0)))
    }
}

// impl Create for RasterPipeline {
//     fn create(resources: &Resources, desc: Self::Desc<'_>) -> VulkanResult<Res<Self>> {
//         let ctx = &resources.ctx;

//         let color_blend = vk::PipelineColorBlendAttachmentState::default()
//             .color_write_mask(vk::ColorComponentFlags::R | vk::ColorComponentFlags::G | vk::ColorComponentFlags::B | vk::ColorComponentFlags::A)
//             .blend_enable(false);

//         let mut bind: vk::VertexInputBindingDescription = vk::VertexInputBindingDescription::default();

//         let mut stride = 0;
//         for ty in &desc.vertex_attributes {
//             match ty {
//                 ShaderType::Float => {
//                     stride += 4;
//                 },
//                 ShaderType::Float4 => {
//                     stride += 16;
//                 },
//                 ShaderType::Float3 => {
//                     stride += 12;
//                 },
//                 _ => todo!(),
//             }
//         }

//         bind = bind
//             .binding(0)
//             .input_rate(vk::VertexInputRate::VERTEX)
//             .stride(stride);

//         let mut attrs: Vec<vk::VertexInputAttributeDescription> = vec![];

//         let mut offset = 0;
//         let mut location = 0;

//         for ty in &desc.vertex_attributes {
//             match ty {
//                 ShaderType::Float => {
//                     attrs.push(
//                         vk::VertexInputAttributeDescription::default()
//                             .binding(0)
//                             .format(vk::Format::R32_SFLOAT)
//                             .location(location as u32)
//                             .offset(offset),
//                     );
//                     offset += 4;
//                     location += 1;
//                 },
//                 ShaderType::Float3 => {
//                     attrs.push(
//                         vk::VertexInputAttributeDescription::default()
//                             .binding(0)
//                             .format(vk::Format::R32G32B32_SFLOAT)
//                             .location(location as u32)
//                             .offset(offset),
//                     );
//                     // 3 float * 4 bytes = 12 bytes
//                     offset += 12;
//                     location += 1;
//                 },
//                 _ => todo!(),
//             }
//         }

//         let binding = [bind];

//         let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::default()
//             .vertex_binding_descriptions(&binding)
//             .vertex_attribute_descriptions(&attrs);

//         let layout = PipelineLayoutBuilder::new(&ctx.device)
//             .set_layouts(vec![])
//             .push_constant(vec![vk::PushConstantRange::default()
//                 .offset(0)
//                 .size(128)
//                 .stage_flags(vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT)])
//             .build()?;

//         // let pipeline = GraphicsPipelineBuilder::new(&ctx.device)
//         //     .vertex_shader(reflection.vertex.as_ref().unwrap().raw)
//         //     .fragment_shader(reflection.fragment.as_ref().unwrap().raw)
//         //     .render_pass(ctx.window.read().unwrap().render_pass.raw)
//         //     .pipeline_layout(layout.raw)
//         //     .viewport(vec![vk::Viewport::default()
//         //         .x(0.0)
//         //         .y(0.0)
//         //         .width(resolution.width as f32)
//         //         .height(resolution.height as f32)
//         //         .min_depth(0.0)
//         //         .max_depth(1.0)])
//         //     .scissors(vec![
//         //         vk::Rect2D::default()
//         //             .offset(vk::Offset2D { x: 0, y: 0 })
//         //             .extent(resolution)])
//         //     .input_assembly(
//         //         vk::PipelineInputAssemblyStateCreateInfo::default()
//         //             .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
//         //             .primitive_restart_enable(false),
//         //     )
//         //     .rasterization(
//         //         vk::PipelineRasterizationStateCreateInfo::default()
//         //             .depth_clamp_enable(false)
//         //             .rasterizer_discard_enable(false)
//         //             .polygon_mode(vk::PolygonMode::FILL)
//         //             .line_width(1.0)
//         //             .cull_mode(vk::CullModeFlags::NONE)
//         //             .front_face(vk::FrontFace::CLOCKWISE)
//         //             .depth_bias_enable(false),
//         //     )
//         //     .multisampling(
//         //         vk::PipelineMultisampleStateCreateInfo::default()
//         //             .sample_shading_enable(false)
//         //             .rasterization_samples(vk::SampleCountFlags::TYPE_1),
//         //     )
//         //     .color_blending(
//         //         vk::PipelineColorBlendStateCreateInfo::default()
//         //             .logic_op_enable(false)
//         //             .logic_op(vk::LogicOp::COPY)
//         //             .attachments(&[color_blend]),
//         //     )
//         //     .dynamic_state(vec![vk::DynamicState::VIEWPORT,
//         // vk::DynamicState::SCISSOR])     .vertex_input_info(vertex_input_info)
//         //     .build()?;

//         todo!()
//     }

//     type Desc<'a> = RasterPipelineDesc;
// }
