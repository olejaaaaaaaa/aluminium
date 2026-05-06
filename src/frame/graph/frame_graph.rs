use std::any::Any;
use std::sync::Arc;

use ash::vk::{self, ClearValue, ComponentMapping};
use slotmap::Key;
use tracing::{debug, error, info, trace};

use crate::core::{
    CommandPool, CommandPoolBuilder, DescriptorPoolBuilder, DescriptorSetLayout,
    DescriptorSetLayoutBuilder, Device, FrameBuffer, FrameBufferBuilder, ImageBuilder,
    ImageViewBuilder, SwapchainError, VulkanError, VulkanResult,
};
use crate::render_context::RenderContext;
use crate::resources::{Resources, TransientTextureDesc};
use crate::{
    FrameGraphResources, FrameScope, Pass, PassContext, RasterPass, Resolution, RuntimeData,
    ShaderStage, StaticData, TextureFormat,
};

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
        trace!("topological sorted: {:?}", sorted_indices);

        scope.execution_order = sorted_indices;

        for i in &mut scope.passes {
            match i {
                Pass::Raster(pass) => {
                    let mut pool_sizes = vec![];
                    let mut bindings = vec![];

                    for (_, location) in &pass.read_storage_buffers {
                        pool_sizes.push(
                            vk::DescriptorPoolSize::default()
                                .descriptor_count(1)
                                .ty(vk::DescriptorType::STORAGE_BUFFER),
                        );

                        let flags = match location.stage {
                            ShaderStage::Fragment => vk::ShaderStageFlags::FRAGMENT,
                            ShaderStage::Vertex => vk::ShaderStageFlags::VERTEX,
                        };

                        bindings.push(
                            vk::DescriptorSetLayoutBinding::default()
                                .binding(location.binding)
                                .descriptor_count(1)
                                .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                                .stage_flags(flags),
                        );
                    }

                    for (handle, location) in &pass.read_textures {
                        pool_sizes.push(
                            vk::DescriptorPoolSize::default()
                                .descriptor_count(1)
                                .ty(vk::DescriptorType::SAMPLED_IMAGE),
                        );

                        let flags = match location.stage {
                            ShaderStage::Fragment => vk::ShaderStageFlags::FRAGMENT,
                            ShaderStage::Vertex => vk::ShaderStageFlags::VERTEX,
                        };

                        bindings.push(
                            vk::DescriptorSetLayoutBinding::default()
                                .binding(location.binding)
                                .descriptor_count(1)
                                .descriptor_type(vk::DescriptorType::SAMPLED_IMAGE)
                                .stage_flags(flags),
                        );

                        let texture = resources.create_transient(
                            ctx,
                            TransientTextureDesc {
                                name: "debug",
                                format: TextureFormat::Color,
                                resolution: crate::Resolution::FullRes,
                            },
                        )?;

                        scope.resolve_textures.insert(*handle, texture);
                    }

                    let pool = DescriptorPoolBuilder::new(&ctx.device)
                        .max_sets(1)
                        .pool_sizes(&pool_sizes)
                        .build()?;

                    let layout = DescriptorSetLayoutBuilder::new(&ctx.device)
                        .bindings(bindings)
                        .build()?;

                    let set = pool.create_descriptor_set(&ctx.device, &[layout.raw])?[0];

                    let mut buffer_infos = vec![];

                    for (handle, location) in &pass.read_storage_buffers {
                        let buffer = unsafe { &(**handle) };
                        let binding = resources.storage_buffers.read();
                        let buffer = binding.get(buffer.key).unwrap();

                        buffer_infos.push(
                            vk::DescriptorBufferInfo::default()
                                .buffer(buffer.buffer.raw)
                                .offset(0)
                                .range(vk::WHOLE_SIZE),
                        );
                    }

                    let mut image_infos = vec![];

                    for (handle, location) in &pass.read_textures {
                        let texture = scope.resolve_textures.get(handle).unwrap();
                        let binding: parking_lot::lock_api::RwLockReadGuard<
                            '_,
                            parking_lot::RawRwLock,
                            slotmap::SlotMap<
                                crate::resources::ResourceKey,
                                crate::TransientTexture,
                            >,
                        > = resources.transient_textures.read();
                        let texture = binding.get(texture.key).unwrap();

                        image_infos.push(
                            vk::DescriptorImageInfo::default()
                                .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                                .image_view(texture.view.raw),
                        );
                    }

                    let mut writes = vec![];

                    for (idx, (_, location)) in pass.read_storage_buffers.iter().enumerate() {
                        writes.push(
                            vk::WriteDescriptorSet::default()
                                .dst_set(set)
                                .dst_binding(location.binding)
                                .dst_array_element(0)
                                .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                                .buffer_info(std::slice::from_ref(&buffer_infos[idx])),
                        );
                    }

                    for (idx, (_, location)) in pass.read_textures.iter().enumerate() {
                        writes.push(
                            vk::WriteDescriptorSet::default()
                                .dst_set(set)
                                .dst_binding(location.binding)
                                .dst_array_element(0)
                                .descriptor_type(vk::DescriptorType::SAMPLED_IMAGE)
                                .image_info(std::slice::from_ref(&image_infos[idx])),
                        );
                    }

                    unsafe {
                        ctx.device.update_descriptor_sets(&writes, &[]);
                    }

                    std::mem::forget(layout);
                    std::mem::forget(pool);

                    for color in &pass.render_target.colors {
                        if !color.color.id.is_null() {
                            let texture = resources.create_transient(
                                ctx,
                                TransientTextureDesc {
                                    name: "color_target",
                                    format: TextureFormat::Color,
                                    resolution: Resolution::FullRes,
                                },
                            )?;
                            scope.resolve_textures.insert(color.color, texture);
                        }
                    }

                    if let Some(depth) = &pass.render_target.depth {
                        let texture = resources.create_transient(
                            ctx,
                            TransientTextureDesc {
                                name: "depth_target",
                                format: TextureFormat::Depth,
                                resolution: Resolution::FullRes,
                            },
                        )?;
                        scope.resolve_textures.insert(depth.depth, texture);
                    }

                    // for handle in &pass.write_textures {
                    //     let texture =
                    // scope.resolve_textures.get(handle).unwrap();
                    //     let binding = resources.transient_textures.read();
                    //     let texture = binding.get(texture.key).unwrap();

                    //     let texture = resources.create_transient(ctx,
                    // TransientTextureDesc {         name:
                    // "debug",         format:
                    // TextureFormat::Color,
                    //         resolution: crate::Resolution::FullRes
                    //     })?;

                    // }

                    pass.set = Some(set);
                },
            }
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

        // Should I handle this case more correctly?
        if scope.execution_order.is_empty() {
            return Ok(());
        }

        // ------------------------Wait fence + Reset
        // cmd-----------------------------
        let (cmd_buffer, image_index) = {
            let window = ctx
                .window
                .try_read()
                .expect("Error borrowed Window for read");

            let frame_idx = window.current_frame % window.frame_sync.len();
            let sync = &window.frame_sync[frame_idx];

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

            let cmd_buffer = self.cmd_buffers[window.current_frame % ctx.frame_in_flight()];

            unsafe {
                device
                    .reset_command_buffer(cmd_buffer, vk::CommandBufferResetFlags::empty())
                    .map_err(VulkanError::Unknown)?;

                let begin_info = vk::CommandBufferBeginInfo::default()
                    .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

                device
                    .begin_command_buffer(cmd_buffer, &begin_info)
                    .map_err(VulkanError::Unknown)?;
            }

            let image_index = unsafe {
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
                    Err(e) => return Err(VulkanError::Unknown(e)),
                }
            };

            (cmd_buffer, image_index)
        };

        // ------------------------Record Command
        // Buffers-----------------------------
        {
            let window = ctx.window.read();
            let resolution = window.resolution;
            for index in &scope.execution_order {
                let pass = &mut scope.passes[*index];
                match pass {
                    Pass::Raster(ref mut pass) => {
                        let mut clear_values = vec![ClearValue {
                            color: vk::ClearColorValue {
                                float32: [0.0, 0.0, 0.0, 1.0],
                            },
                        }];

                        if let Some(_) = pass.render_target.depth {
                            clear_values.push(ClearValue {
                                depth_stencil: vk::ClearDepthStencilValue {
                                    depth: 1.0,
                                    stencil: 0,
                                },
                            });
                        }

                        let is_backbuffer = pass
                            .render_target
                            .colors
                            .iter()
                            .any(|x| x.color.id.is_null());

                        let frame_buffer: vk::Framebuffer = if is_backbuffer {
                            window.frame_buffers[image_index as usize].raw
                        } else {
                            let mut color_attach = vec![];

                            for i in &pass.render_target.colors {
                                let texture = scope.resolve_textures.get(&i.color).unwrap();
                                let binding = resources.transient_textures.read();
                                let texture = binding.get(texture.key).unwrap();
                                color_attach.push(texture.view.raw);
                            }

                            if let Some(depth) = &pass.render_target.depth {
                                let texture = scope.resolve_textures.get(&depth.depth).unwrap();
                                let binding = resources.transient_textures.read();
                                let texture = binding.get(texture.key).unwrap();
                                color_attach.push(texture.view.raw);
                            }

                            let buffer = FrameBufferBuilder::new(&ctx.device)
                                .attachments(&color_attach)
                                .extent(vk::Extent2D {
                                    width: resolution.width,
                                    height: resolution.height,
                                })
                                .render_pass(window.render_pass.raw)
                                .layers(1)
                                .build()?;

                            let raw = buffer.raw;
                            std::mem::forget(buffer);
                            raw
                        };

                        // ---- БАРЬЕРЫ ДО render pass ----
                        {
                            let mut barriers = vec![];

                            // color render targets: UNDEFINED →
                            // COLOR_ATTACHMENT_OPTIMAL
                            for i in &pass.render_target.colors {
                                if i.color.id.is_null() {
                                    continue;
                                } // пропускаем backbuffer
                                let texture = scope.resolve_textures.get(&i.color).unwrap();
                                let binding = resources.transient_textures.read();
                                let texture = binding.get(texture.key).unwrap();

                                barriers.push(
                                    vk::ImageMemoryBarrier::default()
                                        .old_layout(vk::ImageLayout::UNDEFINED)
                                        .new_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                                        .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                                        .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                                        .image(texture.image.raw)
                                        .subresource_range(vk::ImageSubresourceRange {
                                            aspect_mask: vk::ImageAspectFlags::COLOR,
                                            base_mip_level: 0,
                                            level_count: 1,
                                            base_array_layer: 0,
                                            layer_count: 1,
                                        })
                                        .src_access_mask(vk::AccessFlags::empty())
                                        .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE),
                                );
                            }

                            // depth render target: UNDEFINED →
                            // DEPTH_STENCIL_ATTACHMENT_OPTIMAL
                            if let Some(depth) = &pass.render_target.depth {
                                if let Some(texture_handle) =
                                    scope.resolve_textures.get(&depth.depth)
                                {
                                    let binding = resources.transient_textures.read();
                                    let texture = binding.get(texture_handle.key).unwrap();

                                    barriers.push(
                                        vk::ImageMemoryBarrier::default()
                                            .old_layout(vk::ImageLayout::UNDEFINED)
                                            .new_layout(
                                                vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
                                            )
                                            .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                                            .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                                            .image(texture.image.raw)
                                            .subresource_range(vk::ImageSubresourceRange {
                                                aspect_mask: vk::ImageAspectFlags::DEPTH,
                                                base_mip_level: 0,
                                                level_count: 1,
                                                base_array_layer: 0,
                                                layer_count: 1,
                                            })
                                            .src_access_mask(vk::AccessFlags::empty())
                                            .dst_access_mask(
                                                vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
                                            ),
                                    );
                                }
                            }

                            // read textures: UNDEFINED →
                            // SHADER_READ_ONLY_OPTIMAL
                            for (handle, _) in &pass.read_textures {
                                let texture = scope.resolve_textures.get(handle).unwrap();
                                let binding = resources.transient_textures.read();
                                let texture = binding.get(texture.key).unwrap();

                                barriers.push(
                                    vk::ImageMemoryBarrier::default()
                                        .old_layout(vk::ImageLayout::UNDEFINED)
                                        .new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                                        .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                                        .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                                        .image(texture.image.raw)
                                        .subresource_range(vk::ImageSubresourceRange {
                                            aspect_mask: vk::ImageAspectFlags::COLOR,
                                            base_mip_level: 0,
                                            level_count: 1,
                                            base_array_layer: 0,
                                            layer_count: 1,
                                        })
                                        .src_access_mask(vk::AccessFlags::empty())
                                        .dst_access_mask(vk::AccessFlags::SHADER_READ),
                                );
                            }

                            if !barriers.is_empty() {
                                unsafe {
                                    device.cmd_pipeline_barrier(
                                        cmd_buffer,
                                        vk::PipelineStageFlags::TOP_OF_PIPE,
                                        vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
                                            | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS
                                            | vk::PipelineStageFlags::FRAGMENT_SHADER,
                                        vk::DependencyFlags::empty(),
                                        &[],
                                        &[],
                                        &barriers,
                                    );
                                }
                            }
                        }

                        let render_pass_begin_info = vk::RenderPassBeginInfo::default()
                            .render_pass(window.render_pass.raw)
                            .framebuffer(frame_buffer)
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

                        // ---- БАРЬЕРЫ ПОСЛЕ render pass ----
                        // только для color targets которые следующий пасс будет
                        // читать как sampled
                        {
                            let mut barriers = vec![];

                            for i in &pass.render_target.colors {
                                if i.color.id.is_null() {
                                    continue;
                                } // пропускаем backbuffer
                                let texture = scope.resolve_textures.get(&i.color).unwrap();
                                let binding = resources.transient_textures.read();
                                let texture = binding.get(texture.key).unwrap();

                                barriers.push(
                                    vk::ImageMemoryBarrier::default()
                                        .old_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                                        .new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                                        .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                                        .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                                        .image(texture.image.raw)
                                        .subresource_range(vk::ImageSubresourceRange {
                                            aspect_mask: vk::ImageAspectFlags::COLOR,
                                            base_mip_level: 0,
                                            level_count: 1,
                                            base_array_layer: 0,
                                            layer_count: 1,
                                        })
                                        .src_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE)
                                        .dst_access_mask(vk::AccessFlags::SHADER_READ),
                                );
                            }

                            if !barriers.is_empty() {
                                unsafe {
                                    device.cmd_pipeline_barrier(
                                        cmd_buffer,
                                        vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                                        vk::PipelineStageFlags::FRAGMENT_SHADER,
                                        vk::DependencyFlags::empty(),
                                        &[],
                                        &[],
                                        &barriers,
                                    );
                                }
                            }
                        }
                    },
                }
            }
        }

        unsafe {
            device
                .end_command_buffer(cmd_buffer)
                .map_err(VulkanError::Unknown)?;
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

        trace!(
            "frame={} fif={} cmd_buffers={} idx={}",
            window.current_frame,
            ctx.frame_in_flight(),
            self.cmd_buffers.len(),
            window.current_frame % ctx.frame_in_flight()
        );

        Ok(())
    }

    pub(crate) fn destroy(&mut self, device: &Device) {
        self.cmd_pool.destroy(device);
    }
}
