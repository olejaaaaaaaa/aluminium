use ash::vk;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LogicalDeviceError {
    #[error("Failed create logical devices (Vulkan error: {0:?})")]
    CreateDevice(vk::Result),
}
