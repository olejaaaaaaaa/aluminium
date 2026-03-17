use std::sync::{Arc, LazyLock};

use ash::vk::{self, ClearValue};
mod pass;
pub use pass::*;

pub mod pass_context;
pub use pass_context::*;

mod texture;
use slotmap::SlotMap;
pub use texture::*;

use crate::bindless::Bindless;
use crate::camera::{Camera, CameraData};
use crate::core::{Device, Resolution as _, SwapchainError, VulkanError, VulkanResult};
use crate::frame_values::{FrameData, FrameValues};
use crate::per_frame::CommandPoolPerFrame;
use crate::render_context::RenderContext;
use crate::resources::{Create, Resources};

pub static GLOBAL_BINDLESS_LAYOUT: LazyLock<Vec<vk::DescriptorSetLayoutBinding<'static>>> = LazyLock::new(|| {
    vec![
        // Main Camera
        vk::DescriptorSetLayoutBinding::default()
            .binding(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT),
        // // Frame Values
        // vk::DescriptorSetLayoutBinding::default()
        //     .binding(1)
        //     .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC)
        //     .descriptor_count(1)
        //     .stage_flags(
        //         vk::ShaderStageFlags::VERTEX |
        // vk::ShaderStageFlags::FRAGMENT,     ),
        // SSBO All Transforms
        // vk::DescriptorSetLayoutBinding::default()
        //     .binding(2)
        //     .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
        //     .descriptor_count(10_000 as u32)
        //     .stage_flags(
        //         vk::ShaderStageFlags::VERTEX |
        // vk::ShaderStageFlags::FRAGMENT,     ),
    ]
});

pub struct PerFrameValues {
    camera: Camera,
    frame_values: FrameValues,
}

pub struct FrameGraph {
    ctx: Arc<RenderContext>,
    // bindless: Bindless,
    // frame_values: FrameValues,
    // command_pool: CommandPoolPerFrame,
    external_resources: Arc<Resources>,
    execution_order: Vec<usize>,
    passes: Vec<Pass>,
}

impl FrameGraph {
    /// Create new [`FrameGraph`]
    pub(crate) fn new(ctx: Arc<RenderContext>, resources: Arc<Resources>, camera: &Camera) -> VulkanResult<Self> {
        let bindless = Bindless::new(&ctx.device, &GLOBAL_BINDLESS_LAYOUT)?;

        // bindless.update_buffer_set::<CameraData>(&ctx.device, 0,
        // vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC, camera.buffer());

        // bindless.update_buffer_set(
        //     &ctx.device,
        //     1,
        //     vk::DescriptorType::UNIFORM_BUFFER,
        //     frame_values.buffer.buffers[0].raw,
        //     0,
        //     size_of::<FrameData>() as u64,
        // );

        // bindless.update_buffer_set(
        //     &ctx.device,
        //     2,
        //     vk::DescriptorType::STORAGE_BUFFER,
        //     resources.assets.read().unwrap().transform.buffer.raw,
        //     0,
        //     size_of::<Transform>() as u64 * 10000,
        // );

        Ok(FrameGraph {
            ctx,
            external_resources: resources,
            execution_order: vec![],
            passes: vec![],
        })
    }

    pub fn add_pass<P: Into<Pass>>(&mut self, pass: P) {
        self.passes.push(pass.into());
    }

    pub(crate) fn compile(&mut self) -> VulkanResult<()> {
        profiling::scope!("FrameGraph::compile");

        self.execution_order = (0..self.passes.len()).collect();

        Ok(())
    }

    fn import(&mut self, res: bool, access: vk_sync::AccessType) {}

    fn export(&mut self, access: vk_sync::AccessType) {}

    fn topological_sort(&mut self) {
        profiling::scope!("FrameGraph::topological_sort");
    }

    fn begin_frame(&mut self) {
        profiling::scope!("FrameGraph::begin_frame");
    }

    fn end_frame(&mut self) {
        profiling::scope!("FrameGraph::end_frame");
    }

    // pub(crate) fn acquire_next_image(
    //     ctx: &RenderContext,
    // ) -> VulkanResult<u32> {
    //     let image_index = {
    //         let window = &ctx.window.read().unwrap();
    //         let device = &ctx.device;
    //         let sync = &window.frame_sync
    //             [window.current_frame %
    // window.frame_buffers.len()];

    //         // Wait fence for next frame or skip frame
    //         unsafe {
    //             let wait = device.wait_for_fences(
    //                 &[sync.in_flight_fence.raw],
    //                 true,
    //                 u64::MAX,
    //             );
    //             if let Err(err) = wait {
    //                 log::error!("Error wait for fences: {:?}",
    // err);                 return Err(VulkanError::Swapchain(
    //
    // SwapchainError::SwapchainCreationFailed(err),
    // ));             }
    //             device
    //                 .reset_fences(&[sync.in_flight_fence.raw])
    //                 .expect("Error reset fences");
    //         }

    //         // Get image index or skip a frame
    //         unsafe {
    //             match window.swapchain.loader.acquire_next_image(
    //                 window.swapchain.raw,
    //                 u64::MAX,
    //                 sync.image_available.raw,
    //                 vk::Fence::null(),
    //             ) {
    //                 Ok((index, is_suboptimal)) => {
    //                     if is_suboptimal {
    //                         return Err(VulkanError::Swapchain(
    //
    // SwapchainError::SwapchainSubOptimal,
    // ));                     }
    //                     index
    //                 },
    //                 Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
    //                     return Err(VulkanError::Swapchain(
    //                         SwapchainError::SwapchainOutOfDateKhr,
    //                     ));
    //                 },
    //                 Err(e) => {
    //                     return Err(VulkanError::Unknown(e));
    //                 },
    //             }
    //         }
    //     };

    //     Ok(image_index)
    // }

    pub(crate) fn execute(&mut self) -> VulkanResult<()> {
        Ok(())
    }

    // /// Execute graph
    // pub(crate) fn execute(&mut self) -> VulkanResult<()> {
    //     profile_scope!("FrameGraph::execute");

    //     let image_index = Self::acquire_next_image(&self.ctx)?;

    //     let device = &self.ctx.device;
    //     let window = self.ctx.window.read().unwrap();
    //     let resolution = window.resolution;
    //     let frame_count = window.frame_buffers.len();

    //     // Get Command Buffers or skip frame
    //     let pass_count = self.passes.len();
    //     let command_buffers =
    //         self.command_pool.allocate_cmd_buffers(
    //             device,
    //             image_index,
    //             frame_count,
    //             pass_count,
    //         )?;

    //     // Reset Command Buffers or skip frame
    //     for i in command_buffers {
    //         unsafe {
    //             let reset = device.reset_command_buffer(
    //                 *i,
    //                 vk::CommandBufferResetFlags::empty(),
    //             );
    //             if let Err(err) = reset {
    //                 log::error!(
    //                     "Reset command buffer error: {:?}",
    //                     err
    //                 );
    //                 return Err(VulkanError::Unknown(
    //                     vk::Result::from_raw(0),
    //                 ));
    //             }

    //             let begin_info =
    // vk::CommandBufferBeginInfo::default(             )
    //
    // .flags(vk::CommandBufferUsageFlags::SIMULTANEOUS_USE);

    //             device
    //                 .begin_command_buffer(*i, &begin_info)
    //                 .expect("Error begin command buffer");
    //         }
    //     }

    //     // Record Command Buffer
    //     for i in self.execution_order.drain(..) {
    //         let command_buffer = command_buffers[i];
    //         let pass = &self.passes[i];

    //         // pass.begin_sync(command_buffer);

    //         let pass_ctx = PassContext {
    //             bindless_set: self.bindless.bindless_set(),
    //             resolution,
    //             device: device.raw.clone(),
    //             cbuf: command_buffer,
    //             pipeline: pass.pipeline(&self.resources),
    //             layout: pass.pipeline_layout(&self.resources),
    //             resources: self.resources.clone(),
    //         };

    //         let renderables = self
    //             .resources
    //             .assets
    //             .read()
    //             .unwrap()
    //             .renderable
    //             .get_renderables();

    //         let clear_values = vec![
    //             ClearValue {
    //                 color: vk::ClearColorValue {
    //                     float32: [0.0, 0.0, 0.0, 1.0],
    //                 },
    //             },
    //             ClearValue {
    //                 depth_stencil: vk::ClearDepthStencilValue {
    //                     depth: 1.0,
    //                     stencil: 0,
    //                 },
    //             },
    //         ];

    //         let s = self.resources.low_level.read().unwrap();

    //         let frame_buffer = if pass.is_present() {
    //             &window.frame_buffers[image_index as usize]
    //         } else {
    //             let handle = pass.framebuffer();

    //             s.frame_buffer.get(handle).unwrap()
    //         };

    //         let render_pass_begin_info =
    //             vk::RenderPassBeginInfo::default()
    //                 .render_pass(window.render_pass.raw)
    //                 .framebuffer(frame_buffer.raw)
    //                 .render_area(vk::Rect2D {
    //                     offset: vk::Offset2D { x: 0, y: 0 },
    //                     extent: resolution,
    //                 })
    //                 .clear_values(&clear_values);

    //         unsafe {
    //             device.cmd_begin_render_pass(
    //                 command_buffer,
    //                 &render_pass_begin_info,
    //                 vk::SubpassContents::INLINE,
    //             );
    //         }

    //         //(*pass.execute())(&pass_ctx, &renderables);

    //         unsafe {
    //             device.cmd_end_render_pass(command_buffer);
    //         }

    //         // pass.end_sync(command_buffer);

    //         unsafe {
    //             device
    //                 .end_command_buffer(command_buffer)
    //                 .expect("Error end command buffer");
    //         }
    //     }

    //     drop(window);

    //     let window = &mut self.ctx.window.write().unwrap();
    //     let sync = &window.frame_sync
    //         [window.current_frame % window.frame_buffers.len()];

    //     // Submit
    //     let wait_semaphores = [sync.image_available.raw];
    //     let wait_stages =
    //         [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
    //     let signal_semaphores = [sync.render_finished.raw];

    //     let submit_info = vk::SubmitInfo::default()
    //         .wait_semaphores(&wait_semaphores)
    //         .wait_dst_stage_mask(&wait_stages)
    //         .command_buffers(command_buffers)
    //         .signal_semaphores(&signal_semaphores);

    //     unsafe {
    //         device
    //             .queue_submit(
    //                 self.graphics_queue,
    //                 &[submit_info],
    //                 sync.in_flight_fence.raw,
    //             )
    //             .expect("Error submit commands to queue");
    //     }

    //     // Present
    //     let swapchain = [window.swapchain.raw];
    //     let image_indices = [image_index];

    //     let present_info = vk::PresentInfoKHR::default()
    //         .wait_semaphores(&signal_semaphores)
    //         .swapchains(&swapchain)
    //         .image_indices(&image_indices);

    //     unsafe {
    //         window
    //             .swapchain
    //             .loader
    //             .queue_present(self.graphics_queue, &present_info)
    //             .expect("Error present");
    //     }

    //     // log::debug!(
    //     //     "Image index: {}, Frame: {}",
    //     //     image_index,
    //     //     window.current_frame
    //     // );

    //     self.frame_values.update(
    //         device,
    //         image_index,
    //         window.current_frame as u32,
    //     )?;
    //     self.passes.clear();
    //     window.current_frame += 1;

    //     Ok(())
    // }

    // pub(crate) fn destroy(&mut self, device: &Device) {
    //     self.command_pool.destroy(device);
    // }
}
