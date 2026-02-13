
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

mod command_pool;
pub use command_pool::CommandPoolPerFrame;

use crate::bindless::Bindless;
use crate::core::{
    Device, SwapchainError,
    VulkanError, VulkanResult,
};
use crate::render_context::RenderContext;
use crate::resource_manager::ResourceManager;

pub struct RenderGraph {
    bindless_set: vk::DescriptorSet,
    graphics_queue: vk::Queue,
    command_pool: CommandPoolPerFrame,
    execution_order: Vec<usize>,
    passes: Vec<Pass>,
    pass_descs: Vec<PassDesc>,
}

impl RenderGraph {
    /// Create new [`RenderGraph`]
    pub(crate) fn new(ctx: &RenderContext, bindless: &Bindless) -> VulkanResult<Self> {
        let queue = ctx
            .device
            .queue_pool
            .get_queue(vk::QueueFlags::GRAPHICS)
            .unwrap();

        Ok(RenderGraph {
            bindless_set: bindless.set,
            graphics_queue: queue,
            command_pool: CommandPoolPerFrame::new(&ctx.device)?,
            execution_order: vec![],
            pass_descs: vec![],
            passes: vec![],
        })
    }

    pub fn add_pass<P: Into<PassDesc>>(&mut self, pass: P) {
        self.pass_descs.push(pass.into());
    }

    pub fn compile(
        &mut self,
        ctx: &RenderContext,
        resources: &mut ResourceManager,
    ) -> VulkanResult<()> {
        for desc in self.pass_descs.drain(..) {
            let pass = match desc {
                PassDesc::Present(pass) => {
                    let (pipeline, layout) = resources
                        .low_level
                        .create_raster_pipeline(ctx, &pass.pipeline_desc)?;

                    Pass::Present(PresentPass {
                        reads: pass.reads,
                        pipeline_layout: layout,
                        pipeline,
                        execute_fn: pass.execute_fn,
                    })
                },
            };

            self.passes.push(pass);
        }

        self.execution_order = (0..self.passes.len()).collect();

        Ok(())
    }

    fn topological_sort(&mut self) {
        profile_scope!("RenderGraph::topological_sort");
    }

    /// Execute graph
    pub fn execute(
        &mut self,
        ctx: &mut RenderContext,
        resources: &mut ResourceManager,
    ) -> VulkanResult<()> {
        profile_scope!("RenderGraph::execute");

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
        let frame_count = ctx.window.frame_sync.len();
        let pass_count = self.passes.len();
        let command_buffers =
            &self
                .command_pool
                .allocate_cmd_buffers(&ctx.device, frame_count, pass_count)?[image_index as usize];

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
                    .expect("Error begin command buffer");
            }
        }

        // Record Command Buffer
        for i in self.execution_order.drain(..) {
            let command_buffer = command_buffers[i];
            let pass = &self.passes[i];

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
            .command_buffers(command_buffers)
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

        self.passes.clear();
        window.current_frame += 1;

        Ok(())
    }

    pub fn destroy(&mut self, _device: &Device) {
        // self.command_pool.destroy(device);
    }
}
