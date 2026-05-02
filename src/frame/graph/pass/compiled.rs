use ash::vk;
use crate::frame::PassContext;

pub struct CompiledPass<'frame> {
    execute: Box<dyn FnOnce(&mut PassContext) + Send + 'frame>,
    sync: Box<dyn FnOnce(vk::CommandBuffer)>
}