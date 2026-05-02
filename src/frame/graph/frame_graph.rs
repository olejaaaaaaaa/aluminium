use std::any::Any;
use std::sync::Arc;

use ash::vk::{self, ClearValue, ComponentMapping};
use slotmap::Key;
use tracing::{debug, error, trace};

use crate::{FrameGraphResources, FrameScope, Pass, PassContext, RasterPass, RuntimeData, StaticData, TextureFormat};
use crate::core::{
    CommandPool, CommandPoolBuilder, DescriptorPoolBuilder, DescriptorSetLayout, DescriptorSetLayoutBuilder, Device, ImageBuilder, ImageViewBuilder, SwapchainError, VulkanError, VulkanResult
};
use crate::render_context::RenderContext;
use crate::resources::Resources;

pub struct FrameGraph {
    pub resources: FrameGraphResources,
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
            ctx.frame_in_flight() as u32,
        )?;

        let resources = FrameGraphResources::new();

        Ok(FrameGraph {
            cmd_pool,
            cmd_buffers,
            resources,
        })
    }

    fn topological_sort(dependencies: &[Vec<usize>]) -> Vec<usize> {
        let n = dependencies.len();
        
        let mut in_degree: Vec<usize> = dependencies.iter().map(|d| d.len()).collect();
        
        let mut queue: Vec<usize> = (0..n).filter(|&i| in_degree[i] == 0).collect();
        let mut result = Vec::with_capacity(n);

        while let Some(node) = queue.pop() {
            result.push(node);

            for j in 0..n {
                if dependencies[j].contains(&node) {
                    in_degree[j] -= 1;
                    if in_degree[j] == 0 {
                        queue.push(j);
                    }
                }
            }
        }

        result
    }

    pub(crate) fn compile(
        &mut self,
        scope: &mut FrameScope<'_>,
        ctx: &Arc<RenderContext>,
        resources: &Arc<Resources>,
    ) -> VulkanResult<()> {
        profiling::scope!("FrameGraph::compile");

        let mut dependencies: Vec<Vec<usize>> = vec![vec![]; scope.passes.len()];

       for (i, pass_a) in scope.passes.iter().enumerate() {
            let writes = pass_a.texture_writes();
            for (j, pass_b) in scope.passes.iter().enumerate() {
                if i != j && pass_b.texture_reads().iter().any(|r| writes.contains(r)) {
                    dependencies[j].push(i);
                }
            }
        }

        let sorted_indices = Self::topological_sort(&dependencies);
        debug!("topological sorted: {:?}", sorted_indices);

        scope.execution_order = sorted_indices;

        let pool = DescriptorPoolBuilder::new(&ctx.device)
            .max_sets(1)
            .pool_sizes(&[
                vk::DescriptorPoolSize::default()
                    .descriptor_count(1)
                    .ty(vk::DescriptorType::STORAGE_BUFFER)
            ])
            .build()?;

        let binding = vk::DescriptorSetLayoutBinding::default()
            .binding(0)
            .descriptor_count(1)
            .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
            .stage_flags(vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT);

        let layot = DescriptorSetLayoutBuilder::new(&ctx.device)
            .bindings(vec![
                binding
            ])
            .build()?;

       for i in &mut scope.passes {
            match i {
                Pass::Raster(pass) => {
                    let buffers = unsafe { (*pass.read_storage_buffers[0].0).clone() };
                    let binding = resources.storage_buffers.read();
                    let buffer = binding.get(buffers.key).unwrap();
                    
                    let set = pool.create_descriptor_set(&ctx.device, &[layot.raw])?[0];

                    let buffer_info = vk::DescriptorBufferInfo::default()
                        .buffer(buffer.buffer.raw) 
                        .offset(0)
                        .range(vk::WHOLE_SIZE);

                    let write = vk::WriteDescriptorSet::default()
                        .dst_set(set)
                        .dst_binding(0)        
                        .dst_array_element(0)
                        .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                        .buffer_info(std::slice::from_ref(&buffer_info));

                    unsafe {
                        ctx.device.update_descriptor_sets(&[write], &[]);
                    }

                    pass.set = Some(set);

                    {
                        let binding2 = resources.storage_buffers.read();
                        let buf2 = binding2.get(buffers.key).unwrap();
                        println!("buffer raw handle: {:?}", buf2.buffer.raw);
                        println!("buffer size: {:?}", buf2.buffer.allocation.as_ref().unwrap().size());

                        // и mapped ptr
                        let ptr = buf2.buffer.allocation.as_ref().unwrap()
                            .mapped_ptr().unwrap().as_ptr() as *const f32;
                        unsafe {
                            println!("first f32 in buffer: {}", *ptr);
                    }

                }
                }
            }
       }

       std::mem::forget(layot);
       std::mem::forget(pool);

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

        // Should I handle this case more correctly?
        if scope.execution_order.is_empty() {
            return Ok(());
        }

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

        let cmd_buffer = self.cmd_buffers[image_index as usize];

        // ------------------------Record Command Buffers-----------------------------
        {
            let window = ctx.window.read();
            let resolution = window.resolution;
            for index in scope.execution_order.drain(..) {
                let pass = scope.passes.get_mut(index).unwrap();
                match pass {
                    Pass::Raster(ref mut pass) => {
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
                            external_resources: resources.clone(),
                            static_data: StaticData {
                                device: ctx.device.raw.clone(),
                                bindless: resources.bindless_set(),
                                cbuf: cmd_buffer,
                                //per_frame: resources.per_frame_set(),
                                resolution,
                            },
                            runtime_data: RuntimeData {
                                addition_sets: vec![pass.set.unwrap()],
                                push: None,
                                bind_point: None,
                                layout: None,
                                pipeline: None,
                                viewport: None,
                                scissor: None,
                            },
                        };

                        if let Some(execute) = pass.execute.take() {
                            (execute)(&mut pass_ctx);
                        }

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

        resources.update(image_index);

        window.current_frame = window.current_frame.overflowing_add(1).0;

        Ok(())
    }

    pub(crate) fn destroy(&mut self, device: &Device) {
        self.cmd_pool.destroy(device);
    }
}
