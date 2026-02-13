use ash::vk;
use puffin::profile_scope;

use super::device::Device;
use super::{VulkanError, VulkanResult};

pub struct CommandPool {
    pub raw: vk::CommandPool,
}

impl CommandPool {
    pub fn destroy(&self, device: &Device) {
        unsafe { device.destroy_command_pool(self.raw, None) };
    }

    pub fn create_command_buffers(
        &self,
        device: &Device,
        count: u32,
    ) -> VulkanResult<Vec<vk::CommandBuffer>> {
        profile_scope!("CommandBuffers");

        let create_info = vk::CommandBufferAllocateInfo::default()
            .command_pool(self.raw)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(count);

        let buffers = unsafe {
            device
                .allocate_command_buffers(&create_info)
                .map_err(VulkanError::Unknown)
        }?;

        Ok(buffers)
    }
}

pub struct CommandPoolBuilder<'a> {
    device: &'a Device,
    create_info: vk::CommandPoolCreateInfo<'static>,
}

impl<'a> CommandPoolBuilder<'a> {
    pub fn reset(device: &'a Device) -> Self {
        Self {
            device,
            create_info: vk::CommandPoolCreateInfo::default()
                .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER),
        }
    }

    pub fn build(self) -> VulkanResult<CommandPool> {
        puffin::profile_scope!("CommandPool");

        let pool = unsafe {
            self.device
                .create_command_pool(&self.create_info, None)
                .map_err(VulkanError::Unknown)
        }?;

        Ok(CommandPool { raw: pool })
    }
}
