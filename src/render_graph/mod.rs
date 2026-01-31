use std::path::{Path, PathBuf};
use std::process::Command;
use std::ptr::NonNull;
use std::sync::Arc;
use std::time::Instant;

use ash::vk;
use puffin::{profile_scope, GlobalProfiler};
use slotmap::SlotMap;

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
    CommandPool, CommandPoolBuilder, DescriptorSetLayoutBuilder, Device, GraphicsPipelineBuilder,
    PipelineLayoutBuilder, ShaderBuilder, ShaderError, ShaderModule, SwapchainError, VulkanError,
    VulkanResult,
};
use crate::reflection::PipelineReflection;
use crate::render_context::RenderContext;

pub struct RenderGraph {
    bindless_set: vk::DescriptorSet,
    graphics_queue: vk::Queue,
    command_pool: CommandPool,
    resources: RenderGraphResources,
    passes: Vec<Pass>,
    command_buffers: Vec<Vec<vk::CommandBuffer>>,
    is_compiled: bool,
}

impl RenderGraph {
    /// Create new RenderGraph
    pub fn new(ctx: &RenderContext, bindless: &Bindless) -> VulkanResult<Self> {
        let pool = CommandPoolBuilder::reset(&ctx.device).build()?;
        let queue = ctx
            .device
            .queue_pool
            .get_queue(vk::QueueFlags::GRAPHICS)
            .unwrap();

        Ok(RenderGraph {
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

    pub(crate) fn recreate_transient_resources(&mut self) {}

    fn pipeline_reflection(
        ctx: &RenderContext,
        pass: &mut Pass,
    ) -> VulkanResult<Vec<ShaderModule>> {
        let mut shaders = vec![];

        // if path.starts_with("shaders://") {

        //     let shader_name = path
        //         .file_name()
        //         .ok_or(VulkanError::Shader(ShaderError::ShaderNameNotValidUnicode))?
        //         .to_str()
        //         .unwrap();

        //     println!("Shader: {}", shader_name);

        //     let shader_name_no_ext = shader_name
        //         .trim_end_matches(".vert")
        //         .trim_end_matches(".frag")
        //         .trim_end_matches(".comp")
        //         .trim_end_matches(".hlsl");

        //     let stage = if shader_name.ends_with(".vert") {
        //         "vert"
        //     } else if shader_name.ends_with(".frag") {
        //         "frag"
        //     } else if shader_name.ends_with(".comp") {
        //         "comp"
        //     } else if shader_name.ends_with(".hlsl") {
        //         "hlsl"
        //     } else {
        //         return Err(VulkanError::Shader(ShaderError::ShaderInvalidExtension));
        //     };

        //     let file_name = format!("{}-{}.spv", shader_name_no_ext, stage);

        //     let root = env!("CARGO_MANIFEST_DIR");
        //     let path = Path::new(root).join("shaders/spv/").join(&file_name);

        //     log::debug!("Shader URI: {} -> Path: {:?}", path.display(), path);

        //     let shader = ShaderBuilder::from_file(&ctx.device, true, path)?;
        //     let reflection =
        // PipelineReflection::new_from_u32(&shader.spirv_bytes.unwrap())?;

        //     shaders.push(shader);

        // }

        // if path.starts_with("kajiya-shaders://") {

        //     let shader_path_str = path
        //         .strip_prefix("kajiya-shaders://")
        //         .map_err(|e| {
        //             VulkanError::Shader(ShaderError::ShaderInvalidPath)
        //         })?;

        //     let shader_path = Path::new(shader_path_str);

        //     let shader_name = shader_path
        //         .file_name()
        //         .ok_or(VulkanError::Shader(ShaderError::ShaderNameNotValidUnicode))?
        //         .to_str()
        //         .ok_or(VulkanError::Shader(ShaderError::ShaderNameNotValidUnicode))?;

        //     let shader_name_no_ext = shader_name
        //         .trim_end_matches(".hlsl")
        //         .trim_end_matches(".vert")
        //         .trim_end_matches(".frag")
        //         .trim_end_matches(".comp");

        //     let input_path = if shader_path.is_absolute() {
        //         shader_path.to_path_buf()
        //     } else {
        //         let root = env!("CARGO_MANIFEST_DIR");
        //         Path::new(root).join("kajiya-shaders").join(shader_path)
        //     };

        //     let root = env!("CARGO_MANIFEST_DIR");
        //     let output_path = Path::new(root)
        //         .join("shaders/spv")
        //         .join(format!("{}.spv", shader_name_no_ext));

        //     let target_profile = if shader_name.ends_with("_vs.hlsl") {
        //         "vs_6_6"
        //     } else if shader_name.ends_with("_ps.hlsl") {
        //         "ps_6_6"
        //     } else {
        //         "cs_6_6"
        //     };

        //     log::info!("Compiling shader:");
        //     log::info!("Input:  {}", input_path.display());
        //     log::info!("Output: {}", output_path.display());
        //     log::info!("Profile: {}", target_profile);

        //     let output = Command::new("dxc")
        //         .arg(input_path.to_str().unwrap())
        //         .arg("-spirv")
        //         .arg("-T").arg(target_profile)
        //         .arg("-E").arg("main")
        //         .arg("-I").arg("D:/banana/kajiya-shaders/inc")
        //         .arg("-fspv-target-env=vulkan1.1")
        //         .arg("-fvk-use-dx-layout")
        //         .arg("-WX")
        //         .arg("-Ges")
        //         .arg("-HV").arg("2021")
        //         .arg("-Fo").arg(output_path.to_str().unwrap())
        //         .spawn()
        //         .and_then(|mut e| {
        //             e.wait()
        //         });

        //     log::info!("compiled shader: {:?}", output);
        //     let spirv_bytes = std::fs::read(&output_path).unwrap();
        //     let reflection = PipelineReflection::new_from_u8(&spirv_bytes)?;
        // }

        Ok(shaders)
    }

    pub fn compile(&mut self, ctx: &RenderContext) -> VulkanResult<()> {
        profile_scope!("RenderGraph::compile");

        let push_constant_size = ctx.device.phys_dev.limits().max_push_constants_size;

        for (index, pass) in self.passes.iter_mut().enumerate() {
            log::info!("Pass: {}", index);
            log::info!("Reads: {:?}", pass.reads());
            log::info!("Writes: {:?}", pass.writes());

            for i in pass.writes() {
                match i {
                    RenderGraphResource::RenderTarget { texture, ops } => {
                        let desc = self.resources.get_texture(texture.0);
                        log::info!("Write to RenderTarget: {:?}", desc);
                    },
                    _ => {},
                }
            }

            // let shaders = Self::pipeline_reflection(ctx, pass)?;

            // let color_blend =
            // vk::PipelineColorBlendAttachmentState::default()
            //     .color_write_mask(
            //           vk::ColorComponentFlags::R
            //         | vk::ColorComponentFlags::G
            //         | vk::ColorComponentFlags::B
            //         | vk::ColorComponentFlags::A
            //     )
            //     .blend_enable(false);

            // let vertex_input_info =
            // vk::PipelineVertexInputStateCreateInfo::default();
            // let descriptor_set_layout =
            // DescriptorSetLayoutBuilder::new(&ctx.device).build()?;

            // let layout = PipelineLayoutBuilder::new(&ctx.device)
            //     .set_layouts(vec![])
            //     .push_constant(vec![
            //         vk::PushConstantRange::default()
            //             .offset(0)
            //             .size(push_constant_size)
            //             .stage_flags(vk::ShaderStageFlags::VERTEX |
            // vk::ShaderStageFlags::FRAGMENT)     ])
            //     .build()?;

            // let pipeline = GraphicsPipelineBuilder::new(&ctx.device)
            //     .render_pass(ctx.window.render_pass.raw)
            //     .pipeline_layout(layout.raw)
            //     .viewport(vec![
            //         vk::Viewport::default()
            //             .x(0.0)
            //             .y(0.0)
            //             .width(ctx.window.resolution.width as f32)
            //             .height(ctx.window.resolution.height as f32)
            //             .min_depth(0.0)
            //             .max_depth(1.0)
            //     ])
            //     .scissors(vec![
            //         vk::Rect2D::default()
            //             .offset(vk::Offset2D { x: 0, y: 0 })
            //             .extent(ctx.window.resolution)
            //     ])
            //     .input_assembly(
            //         vk::PipelineInputAssemblyStateCreateInfo::default()
            //             .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
            //             .primitive_restart_enable(false)
            //     )
            //     .rasterization(
            //         vk::PipelineRasterizationStateCreateInfo::default()
            //             .depth_clamp_enable(false)
            //             .rasterizer_discard_enable(false)
            //             .polygon_mode(vk::PolygonMode::FILL)
            //             .line_width(1.0)
            //             .cull_mode(vk::CullModeFlags::NONE)
            //             .front_face(vk::FrontFace::CLOCKWISE)
            //             .depth_bias_enable(false)
            //     )
            //     .multisampling(
            //         vk::PipelineMultisampleStateCreateInfo::default()
            //             .sample_shading_enable(false)
            //             .rasterization_samples(vk::SampleCountFlags::TYPE_1)
            //     )
            //     .color_blending(
            //         vk::PipelineColorBlendStateCreateInfo::default()
            //             .logic_op_enable(false)
            //             .logic_op(vk::LogicOp::COPY)
            //             .attachments(&[color_blend])
            //     )
            //     .dynamic_state(vec![
            //         vk::DynamicState::VIEWPORT,
            //         vk::DynamicState::SCISSOR
            //     ])
            //     .vertex_input_info(vertex_input_info)
            //     .build()?;
        }

        self.topological_sort();
        self.is_compiled = true;

        Ok(())
    }

    pub fn add_pass<P: Into<Pass>>(&mut self, pass: P) {
        self.passes.push(pass.into());
        self.is_compiled = false;
    }

    pub fn execute(&mut self, ctx: &mut RenderContext) -> VulkanResult<()> {
        profile_scope!("RenderGraph::execute");

        if !self.is_compiled {
            self.compile(ctx)?;
        }

        let window = &mut ctx.window;
        let sync = &window.frame_sync[window.current_frame % window.frame_buffers.len()];

        let device = &ctx.device.device;

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
        let image_index = unsafe {
            match window.swapchain.loader.acquire_next_image(
                window.swapchain.raw,
                u64::MAX,
                sync.image_available.raw,
                vk::Fence::null(),
            ) {
                Ok((index, _)) => index,
                Err(vk::Result::SUBOPTIMAL_KHR) => {
                    return Err(VulkanError::Swapchain(SwapchainError::SwapchainSubOptimal));
                },
                Err(e) => {
                    return Err(VulkanError::Unknown(e));
                },
            }
        };

        log::info!(
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

// impl RenderGraph {

//     /// Destroy all resources RenderGraph
//     pub fn destroy(&mut self, device: &Device) {

//         unsafe { device.device_wait_idle().expect("Error wait idle"); }

//         self.command_pool.destroy(device);

//         for (_, frame) in &mut self.resources.frame_buffer {
//             frame.frame.destroy(device);
//             frame.image_view.destroy(device);
//             frame.image.destory(device);
//             frame.sampler.destroy(device);
//         }

//         // for i in &self.passes {
//         //     // unsafe {
//         //     //     device.destroy_pipeline_layout(i.layout.unwrap(),
// None);         //     //     device.destroy_pipeline(i.pipeline, None);
//         //     // };
//         // }
//     }

//     pub fn execute(&self, ctx: &mut RenderContext, winit_window:
// &winit::window::Window, mesh: &MeshCollection, scene: &Scene) -> Vec<(String,
// u128)> {

//         puffin::profile_scope!("RenderGraph::execute");
//         puffin::GlobalProfiler::lock().new_frame();

//         let window = &mut ctx.window;

//         // Get current frame sync object
//         let sync = &window.frame_sync[window.current_frame %
// window.frame_buffers.len()];

//         let device = &ctx.device.device;

//         // Wait fence for next frame
//         unsafe {
//             device.wait_for_fences(&[sync.in_flight_fence.raw], true,
// u64::MAX).expect("Error wait for fences");
// device.reset_fences(&[sync.in_flight_fence.raw]).expect("Error reset
// fences");         }

//         // Get image index or skip a frame
//         let (image_index, _) = unsafe {
//             match window.swapchain.loader.acquire_next_image(
//                 window.swapchain.raw,
//                 u64::MAX,
//                 sync.image_available.raw,
//                 vk::Fence::null()
//             ) {
//                 Ok((index, _)) => {
//                     (index, false)
//                 },
//                 Err(_) => {
//                     return vec![];
//                 }
//             }
//         };

//         let cmd_buffers = &self.cmd_bufs[image_index as usize];

//         for &cbuf in cmd_buffers {
//             unsafe {
//                 device.reset_command_buffer(cbuf,
// vk::CommandBufferResetFlags::empty()).expect("Error reset command buffer");
//                 let begin_info =
// vk::CommandBufferBeginInfo::default().
// flags(vk::CommandBufferUsageFlags::SIMULTANEOUS_USE);
// device.begin_command_buffer(cbuf, &begin_info).expect("Error begin command
// buffer")             }
//         }

//         let mut v = vec![];

//         for index in 0..self.passes.len() {

//             let time = Instant::now();

//             let pass = &self.passes[index];
//             let cbuf = cmd_buffers[index];

//             let clear_values = [
//                 vk::ClearValue {
//                     color: vk::ClearColorValue {
//                         float32: [5.0/255.0, 5.0/255.0, 5.0/255.0, 1.0],
//                     },
//                 },
//                 vk::ClearValue {
//                     depth_stencil: vk::ClearDepthStencilValue {
//                         depth: 1.0,
//                         stencil: 0,
//                     },
//                 },
//             ];

//             let frame_buffer = match pass.target {
//                 RenderTarget::FrameBuffer(handle) => {
//                     &self.resources.frame_buffer.get(handle).expect("Not
// found frame buffer").frame                 }
//                 RenderTarget::Swapchain => {
//                     &window.frame_buffers[image_index as usize]
//                 }
//             };

//             let render_pass_begin_info = vk::RenderPassBeginInfo::default()
//                 .render_pass(window.render_pass.raw)
//                 .framebuffer(frame_buffer.raw)
//                 .render_area(vk::Rect2D {
//                     offset: vk::Offset2D { x: 0, y: 0 },
//                     extent: window.resolution,
//                 })
//                 .clear_values(&clear_values);

//             unsafe {
//                 device.cmd_begin_render_pass(cbuf,
// &render_pass_begin_info,vk::SubpassContents::INLINE)             };

//             {

//                 let mut sets = vec![];

//                 for i in &pass.bind_sets {
//                     let set =
// *self.resources.set.get(i.set_handle).expect("Not found descriptor set");
//                     sets.push(set);
//                 }

//                 let pass_ctx = PassContext {
//                     window: winit_window,
//                     mesh,
//                     bindless: self.bindless,
//                     device,
//                     sets,
//                     resolution: window.resolution,
//                     cbuf,
//                     pipeline: pass.pipeline,
//                     layout: pass.layout,
//                 };

//                 let nope = vec![];
//                 let renderables =
// scene.renderables.get(&pass.name).unwrap_or(&nope);
// (pass.execute)(&pass_ctx, &renderables);             }

//             unsafe { device.cmd_end_render_pass(cbuf) };

//             match pass.target {
//                 RenderTarget::FrameBuffer(handle) => {

//                     let frame_buffer =
// self.resources.frame_buffer.get(handle).expect("Frame buffer not found");

//                     let image_barrier = vk::ImageMemoryBarrier::default()
//                         .old_layout(vk::ImageLayout::PRESENT_SRC_KHR)
//
// .new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
// .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
// .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
// .image(frame_buffer.image.raw)
// .subresource_range(vk::ImageSubresourceRange {
// aspect_mask: vk::ImageAspectFlags::COLOR,
// base_mip_level: 0,                             level_count: 1,
//                             base_array_layer: 0,
//                             layer_count: 1,
//                         });

//                     unsafe {
//                         device.cmd_pipeline_barrier(
//                         cbuf,
//                         vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
//                         vk::PipelineStageFlags::FRAGMENT_SHADER,
//                         vk::DependencyFlags::empty(),
//                         &[],
//                         &[],
//                         &[image_barrier]
//                         )
//                     };
//                 }
//                 _ => {}
//             }

//             unsafe { device.end_command_buffer(cbuf).expect("Error end
// command buffer") };             v.push((pass.name.clone(),
// time.elapsed().as_millis()));

//         }

//         let sync = &window.frame_sync[window.current_frame %
// window.frame_buffers.len()];         let wait_semaphores =
// [sync.image_available.raw];         let wait_stages =
// [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];         let
// signal_semaphores = [sync.render_finished.raw];

//         let submit_info = vk::SubmitInfo::default()
//             .wait_semaphores(&wait_semaphores)
//             .wait_dst_stage_mask(&wait_stages)
//             .command_buffers(&cmd_buffers)
//             .signal_semaphores(&signal_semaphores);

//         unsafe {
//             device.queue_submit(self.graphics_queue, &[submit_info],
// sync.in_flight_fence.raw).expect("Error submit commands to queue")         };

//         let binding1 = [window.swapchain.raw];
//         let binding = [image_index];

//         let present_info = vk::PresentInfoKHR::default()
//             .wait_semaphores(&signal_semaphores)
//             .swapchains(&binding1)
//             .image_indices(&binding);

//         unsafe { window.swapchain.loader.queue_present(self.graphics_queue,
// &present_info).expect("Error present") };

//         window.current_frame += 1;
//         v
//     }
// }

// pub struct RenderGraphBuilder {
//     frame_buffer: SlotMap<FrameBufferHandle, FrameDesc>,
//     set_layout: SlotMap<DescriptorSetHandle, DescriptorSetLayout>,
//     binds: Vec<Binding>,
//     passes: Vec<Pass>
// }

// pub struct BindSet {
//     pub set: u32,
//     pub set_handle: DescriptorSetHandle,
// }

// pub struct Binding {
//     bind: u32,
//     set: DescriptorSetHandle,
//     resource: GraphResource
// }

// impl RenderGraphBuilder {

//     pub fn new() -> Self {
//         RenderGraphBuilder {
//             passes: vec![],
//             binds: vec![],
//             set_layout: SlotMap::with_key(),
//             frame_buffer: SlotMap::with_key(),
//         }
//     }

//     pub fn add_pass(&mut self, pass: Pass) {
//         self.passes.push(pass);
//     }

//     pub fn create_frame_buffer(&mut self, desc: FrameDesc) ->
// FrameBufferHandle {         self.frame_buffer.insert(desc)
//     }

//     pub fn create_descriptor_set(&mut self, set_layout: DescriptorSetLayout)
// -> DescriptorSetHandle {         self.set_layout.insert(set_layout)
//     }

//     pub fn bind_resource_to_set<T: AnyGraphResource>(&mut self, bind: u32,
// set: DescriptorSetHandle, resources: T) {          self.binds.push(Binding {
//             bind,
//             set,
//             resource: resources.resource()
//         });
//     }

//     pub fn recreate_transient_resources(&mut self) {

//     }

//     ///
//     /// Compile into real Render Graph
//     ///
//     /// This is a slow function and should only be executed when adding a new
// pass or removing a pass.     ///
//     /// Create transient resources and other resources
//     ///
//     /// Correct synchronization between rendering passes
//     ///
//     pub fn compile(self, ctx: &RenderContext, desc: &DescriptorManager,
// bindless: vk::DescriptorSet) -> RenderGraph {

//         profile_scope!("RenderGraphBuilder::compile");

//         let mut res = RenderGraphResources::new();

//         for (handle, desc) in self.frame_buffer {

//             let image = ImageBuilder::new_2d(
//                 &ctx.device,
//                 desc.format,
//                 vk::Extent2D { width: desc.width, height: desc.height }
//             )
//             .usage(desc.usage)
//             .build()
//             .unwrap();

//             let image_view = ImageViewBuilder::new_2d(
//                 &ctx.device,
//                 desc.format,
//                 image.raw
//             )
//             .build()
//             .unwrap();

//             let frame_buffer = FrameBufferBuilder::new(&ctx.device,
// ctx.window.render_pass.raw)                 .add_attachment(image_view.raw)
//                 .add_attachment(ctx.window.depth_view.raw)
//                 .extent(ctx.window.resolution)
//                 .layers(1)
//                 .build()
//                 .unwrap();

//             let sampler =
// SamplerBuilder::default(&ctx.device).build().unwrap();

//             let frame = GraphFrameBuffer {
//                 frame: frame_buffer,
//                 sampler,
//                 image_view,
//                 image
//             };

//             res.frame_buffer.insert(handle, frame);

//         }

//         let mut cmd_bufs =
// Vec::with_capacity(ctx.window.frame_buffers.len());         let pool =
// CommandPoolBuilder::reset(&ctx.device).build().unwrap();

//         for _ in 0..ctx.window.frame_buffers.len() {
//             let buffers = pool.create_command_buffers(&ctx.device,
// self.passes.len() as u32).unwrap();             cmd_bufs.push(buffers);
//         }

//         for (handle, layout) in self.set_layout {
//             let set = desc.create_descriptor_set(&ctx.device,
// &[layout.raw])[0];             res.set.insert(handle, set);
//         }

//         for i in self.binds {

//             match i.resource {
//                 GraphResource::FrameBuffer(handle) => {

//                     let frame_buffer =
// res.frame_buffer.get(handle).expect("Not found Frame Buffer");

//                     let image_info = vk::DescriptorImageInfo::default()
//
// .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
// .image_view(frame_buffer.image_view.raw)
// .sampler(frame_buffer.sampler.raw);

//                     let set = res.set.get(i.set).unwrap();

//                     let bind = &[image_info];

//                     let write = vk::WriteDescriptorSet::default()
//                         .dst_set(*set)
//                         .dst_binding(i.bind)
//
// .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
// .image_info(bind);

//                     unsafe {
//                         ctx.device.raw.update_descriptor_sets(&[write], &[]);
//                     }
//                 }
//             }
//         }

//         let queue =
// ctx.device.queue_pool.get_queue(vk::QueueFlags::GRAPHICS).unwrap();

//         RenderGraph {
//             bindless,
//             graphics_queue: queue,
//             command_pool: pool,
//             cmd_bufs,
//             passes: self.passes,
//             resources: res
//         }
//     }
// }
