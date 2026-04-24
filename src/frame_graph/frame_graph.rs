use std::any::Any;
use std::sync::Arc;

use ash::vk::{self, ClearValue, ComponentMapping};
use tracing::{error, trace};

use crate::TextureFormat;
use crate::core::{
    CommandPool, CommandPoolBuilder, Device, ImageBuilder, ImageViewBuilder, SwapchainError, VulkanError, VulkanResult
};
use crate::frame_graph::{FrameGraphResources, Pass, PassContext};
use crate::frame_scope::FrameScope;
use crate::render_context::RenderContext;
use crate::resources::Resources;

pub struct FrameGraph {
    resources: FrameGraphResources,
    cmd_pool: CommandPool,
    cmd_buffers: Vec<vk::CommandBuffer>,
}

impl FrameGraph {
    /// Create new [`FrameGraph`]
    pub(crate) fn new(ctx: &Arc<RenderContext>) -> VulkanResult<Self> {
        let cmd_pool = CommandPoolBuilder::reset(&ctx.device).build()?;

        let cmd_buffers = cmd_pool.allocate_cmd_buffers(
            &ctx.device,
            vk::CommandBufferLevel::PRIMARY,
            ctx.frame_count() as u32,
        )?;

        let resources = FrameGraphResources::new();

        Ok(FrameGraph {
            cmd_pool,
            cmd_buffers,
            resources,
        })
    }

    pub(crate) fn compile(
        &mut self,
        scope: &mut FrameScope<'_>,
        ctx: &Arc<RenderContext>,
        resources: &Arc<Resources>,
    ) -> VulkanResult<()> {
        profiling::scope!("FrameGraph::compile");

        let index = ctx.window.read().current_frame % ctx.window.read().frame_sync.len();
        println!("index: {}", index);

        for (id, desc) in &scope.resources.textures {

            let extent = match desc.resolution {
                _ => {
                    ctx.window.read().resolution
                }
            };

            let format = match desc.format {
                TextureFormat::D32Sfloat => { vk::Format::D32_SFLOAT },
                TextureFormat::R8g8b8a8Srgb => { vk::Format::R8G8B8A8_SRGB },
                TextureFormat::R8g8b8a8Unorm => { vk::Format::R8G8B8A8_UNORM }
            };

            let image = ImageBuilder::new(&ctx.device)
                .usage(vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::SAMPLED)
                .array_layers(1)
                .extent(extent.into())
                .format(format)
                .image_type(vk::ImageType::TYPE_2D)
                .build()?;

            let image_view = ImageViewBuilder::new(&ctx.device)
                .image(image.raw)
                .subresource_range(vk::ImageSubresourceRange::default())
                .components(ComponentMapping::default())
                .format(format)
                .view_type(vk::ImageViewType::TYPE_2D)
                .build()?;

            self.resources.transient_textures.insert(id, image_view);
        }

        Ok(())
    }

    pub(crate) fn execute(
        &mut self,
        scope: &mut FrameScope<'_>,
        ctx: &Arc<RenderContext>,
        resources: &Arc<Resources>,
    ) -> VulkanResult<()> {
        profiling::scope!("FrameGraph::execute");
        let queue = ctx.device.queue_pool.get_present().unwrap();
        let device = &ctx.device;

        // ------------------------Acquire Next Image-----------------------------
        let image_index = {
            let window = &ctx
                .window
                .try_read()
                .expect("Error borrowed Window for read");
            let sync = &window.frame_sync[window.current_frame % window.frame_sync.len()];

            // Wait fence for next frame or skip frame
            unsafe {
                let wait = device.wait_for_fences(&[sync.in_flight_fence.raw], true, u64::MAX);
                if let Err(err) = wait {
                    error!("Error wait for fences: {:?}", err);
                    return Ok(());
                }
                device
                    .reset_fences(&[sync.in_flight_fence.raw])
                    .map_err(VulkanError::Unknown)?;
            }

            // Get image index or skip a frame
            unsafe {
                match window.swapchain.loader.acquire_next_image(
                    window.swapchain.raw,
                    u64::MAX,
                    sync.image_available.raw,
                    vk::Fence::null(),
                ) {
                    Ok((index, _)) => index,
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

        // Setup Backbufer Id -> ImageView
        self.resources.prepare_backbuffers(image_index, scope, ctx);

        
        // allocate FrameBuffers


        let cmd_buffer = self.cmd_buffers[image_index as usize];

        // ------------------------Record Command Buffers-----------------------------
        {
            let window = ctx.window.read();
            let resolution = window.resolution;
            for pass in scope.passes.drain(..) {
                match pass {
                    Pass::Raster(pass) => {
                        let frame_buffer = &window.frame_buffers[image_index as usize];

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

                        unsafe {
                            device
                                .reset_command_buffer(
                                    cmd_buffer,
                                    vk::CommandBufferResetFlags::empty(),
                                )
                                .map_err(VulkanError::Unknown)?;

                            let begin_info = vk::CommandBufferBeginInfo::default()
                                .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

                            device
                                .begin_command_buffer(cmd_buffer, &begin_info)
                                .map_err(VulkanError::Unknown)?;
                        }

                        let render_pass_begin_info = vk::RenderPassBeginInfo::default()
                            .render_pass(window.render_pass.raw)
                            .framebuffer(frame_buffer.raw)
                            .render_area(vk::Rect2D {
                                offset: vk::Offset2D { x: 0, y: 0 },
                                extent: vk::Extent2D {
                                    width: resolution.width,
                                    height: resolution.height,
                                },
                            })
                            .clear_values(&clear_values);

                        unsafe {
                            device.cmd_begin_render_pass(
                                cmd_buffer,
                                &render_pass_begin_info,
                                vk::SubpassContents::INLINE,
                            );
                        }

                        let mut pass_ctx = PassContext {
                            push: None,
                            per_frame_set: resources.per_frame_set(),
                            layout: None,
                            external_resources: resources.clone(),
                            resolution,
                            device: ctx.device.raw.clone(),
                            cbuf: cmd_buffer,
                        };

                        (pass.execute)(&mut pass_ctx);

                        unsafe {
                            device.cmd_end_render_pass(cmd_buffer);
                        }

                        unsafe {
                            device
                                .end_command_buffer(cmd_buffer)
                                .map_err(VulkanError::Unknown)?;
                        }
                    },
                }
            }
        }

        let mut window = ctx.window.try_write().expect("Window already borrowed");

        let sync = &window.frame_sync[window.current_frame % window.frame_sync.len()];

        // -----------------------Submit-----------------------------
        let wait_semaphores = [sync.image_available.raw];
        let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let signal_semaphores = [sync.render_finished.raw];

        let binding = [cmd_buffer];

        let submit_info = vk::SubmitInfo::default()
            .wait_semaphores(&wait_semaphores)
            .wait_dst_stage_mask(&wait_stages)
            .command_buffers(&binding)
            .signal_semaphores(&signal_semaphores);

        unsafe {
            profiling::scope!("vkQueueSubmit");
            device
                .queue_submit(queue.raw, &[submit_info], sync.in_flight_fence.raw)
                .map_err(VulkanError::Unknown)?;
        }

        // -----------------------Present-----------------------------
        let swapchain = [window.swapchain.raw];
        let image_indices = [image_index];

        let present_info = vk::PresentInfoKHR::default()
            .wait_semaphores(&signal_semaphores)
            .swapchains(&swapchain)
            .image_indices(&image_indices);

        unsafe {
            profiling::scope!("vkQueuePresent");
            window
                .swapchain
                .loader
                .queue_present(queue.raw, &present_info)
                .map_err(VulkanError::Unknown)?;
        }

        trace!(
            image_index = ?image_index,
            current_frame = ?window.current_frame
        );

        resources.update(0);

        window.current_frame += 1;

        Ok(())
    }

    pub(crate) fn destroy(&mut self, device: &Device) {
        self.cmd_pool.destroy(device);
    }
}
