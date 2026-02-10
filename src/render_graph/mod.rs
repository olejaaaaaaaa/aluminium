use std::path::{Path, PathBuf};

use ash::vk::{self, ClearValue};
use bytemuck::checked::cast_slice;
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
    AttributeDescriptions, BindingDescriptions, CommandPool, CommandPoolBuilder, DescriptorSetLayoutBuilder, Device, GraphicsPipeline, GraphicsPipelineBuilder, PipelineLayout, PipelineLayoutBuilder, ShaderBuilder, ShaderError, ShaderModule, SwapchainError, Vertex, VulkanError, VulkanResult, load_spv
};
use crate::reflection::PipelineShaderReflection;
use crate::render_context::RenderContext;
use crate::resource_manager::ResourceManager;

pub struct RenderGraph {
    bindless_set: vk::DescriptorSet,
    graphics_queue: vk::Queue,
    command_pool: CommandPool,
    resources: RenderGraphResources,
    passes: Vec<Pass>,
    pass_desc: Vec<PassDesc>,
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
            pass_desc: vec![],
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

    fn shader_reflection(
        ctx: &RenderContext,
        desc: &mut PassDesc,
    ) -> VulkanResult<PipelineShaderReflection> {

        let mut shaders = vec![];

        for source in desc.sources() {
            let shader = match source {
                Source::None => todo!(),
                Source::Path(path_buf) => {
                    let bytecode = load_spv(path_buf);
                    ShaderBuilder::new(&ctx.device)
                        .bytecode(&bytecode)
                        .build()?
                },
                Source::SpirvU32(bytecode) => {
                    ShaderBuilder::new(&ctx.device)
                        .bytecode(bytecode)
                        .build()?
                }
                Source::SpirvU8(bytes) => {
                    ShaderBuilder::new(&ctx.device)
                        .bytecode(cast_slice(bytes))
                        .build()?
                },
            };
            shaders.push(shader);
        }

        let reflection = PipelineShaderReflection::new_from_shaders(shaders)?;

        Ok(reflection)
    }

    fn create_graphics_pipeline(
        ctx: &RenderContext,
        reflection: &PipelineShaderReflection,
    ) -> VulkanResult<(GraphicsPipeline, PipelineLayout)> {
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

        let set_layout = DescriptorSetLayoutBuilder::new(&ctx.device)
            .bindings(vec![
                vk::DescriptorSetLayoutBinding::default()
                    .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                    .descriptor_count(1)
            ])
            .build()?;

        let binds = vec![
            vk::DescriptorSetLayoutBinding::default()
                .binding(0)
                .stage_flags(vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .descriptor_count(1)
        ];

        let descriptor_set_layout = DescriptorSetLayoutBuilder::new(&ctx.device)
            .bindings(binds)
            .build()?;

        let layout = PipelineLayoutBuilder::new(&ctx.device)
            .set_layouts(vec![
                descriptor_set_layout.raw
            ])
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
                .create_command_buffers(&ctx.device, self.pass_desc.len() as u32)?;
            cmd_bufs.push(cmd);
        }

        for (index, mut desc) in &mut self.pass_desc.drain(..).enumerate() {
            let reflection = Self::shader_reflection(ctx, &mut desc)?;
            let pass = Self::create_pipeline(ctx, desc, reflection, resources)?;
            self.passes.push(pass);
            self.execution_order.push(index);
        }

        self.command_buffers = cmd_bufs;

        self.topological_sort();
        self.is_compiled = true;

        Ok(())
    }

    fn create_pipeline(
        ctx: &RenderContext,
        desc: PassDesc,
        reflection: PipelineShaderReflection,
        resources: &mut ResourceManager,
    ) -> VulkanResult<Pass> {

        let compiled_pass = match desc {
            PassDesc::Present(present) => {
                let (pipeline, layout) = Self::create_graphics_pipeline(ctx, &reflection)?;
                let pipeline_handle = resources.add_raster_pipeline(pipeline);
                let layout_handle = resources.add_layout(layout);
                Pass::Present(PresentPass { 
                    reflection,
                    reads: present.reads.clone(), 
                    pipeline_layout: layout_handle, 
                    pipeline: pipeline_handle, 
                    execute_fn: present.execute_fn
                })
            },
            _ => todo!(),
        };

        Ok(compiled_pass)
    }

    /// Add pass to [`RenderGraph`]
    /// 
    /// It is recommended to recompile the graph before rendering.
    pub fn add_pass<P: Into<PassDesc>>(&mut self, pass: P) {
        self.pass_desc.push(pass.into());
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
                    Ok((index, is_suboptimal)) => {
                        if is_suboptimal {
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
                resources,
            };

            let renderables = resources.get_renderables();

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

            let frame_buffer = if pass.is_present() {
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

            (*pass.execute())(&pass_ctx, &renderables);

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
