use ash::vk;

use crate::core::{FrameBuffer, RenderPass};
use crate::frame::PassContext;
use crate::RenderTarget;

pub enum CompiledPass<'frame> {
    Raster(CompiledRasterPass<'frame>),
}

pub struct CompiledRasterPass<'frame> {
    render_pass: RenderPass,
    render_target: RenderTarget,
    frame_buffer: FrameBuffer,
    uniforms: Vec<vk::DescriptorSet>,
    execute: Option<Box<dyn FnOnce(&mut PassContext) + Send + 'frame>>,
    sync: Option<Box<dyn FnOnce(vk::CommandBuffer)>>,
}
