use std::path::{Path, PathBuf};

use ash::vk::{self, ClearValue};
use puffin::profile_scope;

mod pass;
pub use pass::*;

pub mod pass_context;
pub use pass_context::*;

pub mod resources;
pub use resources::*;

mod texture;
pub use texture::*;

use crate::bindless::Bindless;
use crate::core::{
    CommandPool, CommandPoolBuilder, DescriptorSetLayoutBuilder, Device, GraphicsPipeline,
    GraphicsPipelineBuilder, PipelineLayout, PipelineLayoutBuilder, ShaderBuilder, ShaderError, SwapchainError, VulkanError, VulkanResult,
};
use crate::reflection::ShaderReflection;
use crate::render_context::RenderContext;
use crate::resource_manager::ResourceManager;

pub struct RenderGraph {
    bindless_set: vk::DescriptorSet,
    graphics_queue: vk::Queue,
    command_pool: CommandPool,
    resources: RenderGraphResources,
    passes: Vec<Pass>,
    execution_order: Vec<usize>,
    command_buffers: Vec<Vec<vk::CommandBuffer>>,
    is_compiled: bool,
}

impl RenderGraph {
    /// Create new RenderGraph
    pub(crate) fn new(ctx: &RenderContext, bindless: &Bindless) -> VulkanResult<Self> {
        let pool = CommandPoolBuilder::reset(&ctx.device).build()?;
        let queue = ctx
            .device
            .queue_pool
            .get_queue(vk::QueueFlags::GRAPHICS)
            .unwrap();

        Ok(RenderGraph {
            execution_order: vec![],
            bindless_set: bindless.set,
            graphics_queue: queue,
            command_pool: pool,
            resources: RenderGraphResources::new(),
            passes: vec![],
            command_buffers: vec![],
            is_compiled: false,
        })
    }

    pub fn create_texture(&mut self, desc: TextureDesc) -> TextureHandle {
        self.resources.registry_texture(desc)
    }

    fn topological_sort(&mut self) {
        profile_scope!("RenderGraph::topological_sort");
    }

    pub(crate) fn recreate_transient_resources(&mut self, _width: u32, _height: u32) {
        for (_handle, desc) in self.resources.textures.iter() {
            if desc.usage == TextureUsage::Transient {}
        }
    }

    fn resolve_path(path: &PathBuf) -> VulkanResult<PathBuf> {
        const SHADER_PREFIX: &str = "shaders://";

        if path.starts_with(SHADER_PREFIX) {
            let extenion = path.extension().unwrap().to_str().unwrap();

            let shader_name = path
                .file_name()
                .ok_or(VulkanError::Shader(ShaderError::ShaderNameNotValidUnicode))?
                .to_str()
                .ok_or(VulkanError::Shader(ShaderError::ShaderNameNotValidUnicode))?;

            let shader_name_no_ext = shader_name
                .trim_end_matches(".vert")
                .trim_end_matches(".frag")
                .trim_end_matches(".comp")
                .trim_end_matches(".hlsl");

            let file_name = format!("{}-{}.spv", shader_name_no_ext, extenion);

            let root = env!("CARGO_MANIFEST_DIR");
            let path = Path::new(root).join("shaders/spv/").join(&file_name);

            log::info!("Shader URI: {} -> Path: {:?}", path.display(), path);

            return Ok(path);
        }

        return Err(VulkanError::Unknown(vk::Result::from_raw(0)));
    }

    fn shader_reflection(
        ctx: &RenderContext,
        pass: &mut Pass,
    ) -> VulkanResult<Vec<ShaderReflection>> {
        pass.shaders()
            .iter()
            .map(|shader_path| {
                let path = Self::resolve_path(shader_path)?;

                let shader = ShaderBuilder::new(&ctx.device)
                    .file_path(path)
                    .save_bytecode()
                    .build()?;

                let reflection = ShaderReflection::new_from_shader(&shader)?;

                Ok(reflection)
            })
            .collect()
    }

    fn create_graphics_pipeline(
        ctx: &RenderContext,
        shader_reflection: &Vec<ShaderReflection>,
    ) -> VulkanResult<(GraphicsPipeline, PipelineLayout)> {
        let color_blend = vk::PipelineColorBlendAttachmentState::default()
            .color_write_mask(
                vk::ColorComponentFlags::R
                    | vk::ColorComponentFlags::G
                    | vk::ColorComponentFlags::B
                    | vk::ColorComponentFlags::A,
            )
            .blend_enable(false);

        let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::default();
        let _descriptor_set_layout = DescriptorSetLayoutBuilder::new(&ctx.device).build()?;

        // let mut vertex_attribute_desc = vec![];

        let (vertex_shader, fragment_shader) =
            if shader_reflection[0].shader_stage == naga::ShaderStage::Vertex {
                (&shader_reflection[0], &shader_reflection[1])
            } else {
                (&shader_reflection[1], &shader_reflection[1])
            };

        // for i in &vertex_shader.vertex_inputs {
        //     let desc = vk::VertexInputAttributeDescription::default()
        //         .binding(0)
        //         .location(i.location)
        //         .format(i.format)
        //         .offset(i.offset);

        //     vertex_attribute_desc.push(desc);
        // }

        // println!("Vertex Attrib: {:?}", vertex_attribute_desc);

        // vertex_input_info =
        // vertex_input_info.vertex_attribute_descriptions(&vertex_attribute_desc);

        // let vertex_binding_desc = vec![
        //     vk::VertexInputBindingDescription::default()
        //         .binding(0)
        //         .stride(std::mem::size_of::<Vertex>() as u32)
        //         .input_rate(vk::VertexInputRate::VERTEX),
        // ];

        // vertex_input_info =
        // vertex_input_info.vertex_binding_descriptions(&vertex_binding_desc);

        let layout = PipelineLayoutBuilder::new(&ctx.device)
            .set_layouts(vec![])
            .push_constant(vec![vk::PushConstantRange::default()
                .offset(0)
                .size(128)
                .stage_flags(
                    vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                )])
            .build()?;

        let pipeline = GraphicsPipelineBuilder::new(&ctx.device)
            .vertex_shader(vertex_shader.shader)
            .fragment_shader(fragment_shader.shader)
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

        Ok((pipeline, layout))
    }

    pub fn compile(
        &mut self,
        ctx: &RenderContext,
        resources: &mut ResourceManager,
    ) -> VulkanResult<()> {
        profile_scope!("RenderGraph::compile");

        let mut cmd_bufs = vec![];

        for _ in 0..ctx.window.frame_sync.len() {
            let cmd = self
                .command_pool
                .create_command_buffers(&ctx.device, self.passes.len() as u32)?;
            cmd_bufs.push(cmd);
        }

        for (index, pass) in &mut self.passes.iter_mut().enumerate() {
            let shader_reflections = Self::shader_reflection(ctx, pass)?;
            if let Err(err) = Self::create_pipeline(ctx, pass, &shader_reflections, resources) {
                log::error!("Error create pipeline with error: {:?}", err);
            } else {
                self.execution_order.push(index);
            }
        }

        self.command_buffers = cmd_bufs;

        self.topological_sort();
        self.is_compiled = true;

        Ok(())
    }

    fn create_pipeline(
        ctx: &RenderContext,
        pass: &mut Pass,
        shader_reflections: &Vec<ShaderReflection>,
        resources: &mut ResourceManager,
    ) -> VulkanResult<()> {
        match pass {
            Pass::Raster(raster) => {
                let (pipeline, layout) = Self::create_graphics_pipeline(ctx, &shader_reflections)?;
                let pipeline_handle = resources.add_raster_pipeline(pipeline);
                let layout_handle = resources.add_layout(layout);
                raster.pipeline.as_mut().unwrap().pipeline = pipeline_handle;
                raster.pipeline.as_mut().unwrap().pipeline_layout = layout_handle;
            },

            Pass::Present(raster) => {
                let (pipeline, layout) = Self::create_graphics_pipeline(ctx, &shader_reflections)?;
                let pipeline_handle = resources.add_raster_pipeline(pipeline);
                let layout_handle = resources.add_layout(layout);
                raster.pipeline.as_mut().unwrap().pipeline = pipeline_handle;
                raster.pipeline.as_mut().unwrap().pipeline_layout = layout_handle;
            },

            _ => {},
        }

        Ok(())
    }

    /// Add pass to [`RenderGraph`]
    /// 
    /// It is recommended to recompile the graph before rendering.
    pub fn add_pass<P: Into<Pass>>(&mut self, pass: P) {
        self.passes.push(pass.into());
        self.is_compiled = false;
    }

    /// Execute graph
    pub fn execute(
        &mut self,
        ctx: &mut RenderContext,
        resources: &mut ResourceManager,
    ) -> VulkanResult<()> {
        profile_scope!("RenderGraph::execute");

        if !self.is_compiled {
            self.compile(ctx, resources)?;
        }

        let device = &ctx.device.device;

        let resolution = vk::Extent2D {
            width: ctx.window.resolution.width,
            height: ctx.window.resolution.height,
        };

        let image_index = {
            let window = &mut ctx.window;
            let sync = &window.frame_sync[window.current_frame % window.frame_buffers.len()];

            // Wait fence for next frame or skip frame
            unsafe {
                let wait = device.wait_for_fences(&[sync.in_flight_fence.raw], true, u64::MAX);
                if let Err(err) = wait {
                    log::error!("Error wait for fences: {:?}", err);
                    return Ok(());
                }
                device
                    .reset_fences(&[sync.in_flight_fence.raw])
                    .expect("Error reset fences");
            }

            // Get image index or skip a frame
            unsafe {
                match window.swapchain.loader.acquire_next_image(
                    window.swapchain.raw,
                    u64::MAX,
                    sync.image_available.raw,
                    vk::Fence::null(),
                ) {
                    Ok((index, is_not_optimal)) => {
                        if is_not_optimal {
                            return Err(VulkanError::Swapchain(
                                SwapchainError::SwapchainSubOptimal,
                            ));
                        }
                        index
                    },
                    Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                        return Err(VulkanError::Swapchain(
                            SwapchainError::SwapchainOutOfDateKhr,
                        ));
                    },
                    Err(e) => {
                        return Err(VulkanError::Unknown(e));
                    },
                }
            }
        };

        // Get Command Buffers or skip frame
        let command_buffers = &self.command_buffers[image_index as usize];

        // Reset Command Buffers or skip frame
        for i in command_buffers {
            unsafe {
                let reset = device.reset_command_buffer(*i, vk::CommandBufferResetFlags::empty());
                if let Err(err) = reset {
                    log::error!("Reset command buffer error: {:?}", err);
                    return Err(VulkanError::Unknown(vk::Result::from_raw(0)));
                }

                let begin_info = vk::CommandBufferBeginInfo::default()
                    .flags(vk::CommandBufferUsageFlags::SIMULTANEOUS_USE);

                device
                    .begin_command_buffer(*i, &begin_info)
                    .expect("Error begin commandbuffer");
            }
        }

        // Record Command Buffer
        for i in &self.execution_order {
            let command_buffer = command_buffers[*i];
            let pass = &self.passes[*i];

            // pass.begin_sync(command_buffer);

            let pass_ctx = PassContext {
                bindless_set: self.bindless_set,
                resolution,
                device: device.raw.clone(),
                cbuf: command_buffer,
                pipeline: pass.pipeline(resources),
                layout: pass.pipeline_layout(resources),
            };

            let renderables = vec![];

            let clear_values = vec![
                ClearValue {
                    color: vk::ClearColorValue {
                        float32: [0.0, 0.0, 0.0, 1.0],
                    },
                },
                ClearValue {
                    depth_stencil: vk::ClearDepthStencilValue {
                        depth: 1.0,
                        stencil: 0,
                    },
                },
            ];

            let frame_buffer = if pass.is_present_pass() {
                &ctx.window.frame_buffers[image_index as usize]
            } else {
                let handle = pass.framebuffer();
                resources.get_framebuffer(handle).unwrap()
            };

            let window = &ctx.window;

            let render_pass_begin_info = vk::RenderPassBeginInfo::default()
                .render_pass(window.render_pass.raw)
                .framebuffer(frame_buffer.raw)
                .render_area(vk::Rect2D {
                    offset: vk::Offset2D { x: 0, y: 0 },
                    extent: window.resolution,
                })
                .clear_values(&clear_values);

            unsafe {
                device.cmd_begin_render_pass(
                    command_buffer,
                    &render_pass_begin_info,
                    vk::SubpassContents::INLINE,
                );
            }

            (*pass.execute())(&pass_ctx, &renderables)?;

            unsafe {
                device.cmd_end_render_pass(command_buffer);
            }

            // pass.end_sync(command_buffer);

            unsafe {
                device
                    .end_command_buffer(command_buffer)
                    .expect("Error end command buffer");
            }
        }

        let window = &mut ctx.window;
        let sync = &window.frame_sync[window.current_frame % window.frame_buffers.len()];

        // Submit
        let wait_semaphores = [sync.image_available.raw];
        let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let signal_semaphores = [sync.render_finished.raw];

        let submit_info = vk::SubmitInfo::default()
            .wait_semaphores(&wait_semaphores)
            .wait_dst_stage_mask(&wait_stages)
            .command_buffers(&command_buffers)
            .signal_semaphores(&signal_semaphores);

        unsafe {
            device
                .queue_submit(
                    self.graphics_queue,
                    &[submit_info],
                    sync.in_flight_fence.raw,
                )
                .expect("Error submit commands to queue");
        }

        // Present
        let swapchain = [window.swapchain.raw];
        let image_indices = [image_index];

        let present_info = vk::PresentInfoKHR::default()
            .wait_semaphores(&signal_semaphores)
            .swapchains(&swapchain)
            .image_indices(&image_indices);

        unsafe {
            window
                .swapchain
                .loader
                .queue_present(self.graphics_queue, &present_info)
                .expect("Error present");
        }

        log::debug!(
            "Image index: {}, Frame: {}",
            image_index,
            window.current_frame
        );

        window.current_frame += 1;

        Ok(())
    }

    pub fn destroy(&mut self, device: &Device) {
        self.command_pool.destroy(device);
    }
}
