use ash::vk;

use crate::core::{CommandPool, CommandPoolBuilder, Device, VulkanResult};

pub struct CommandPoolPerFrame {
    command_pool: CommandPool,
    command_buffers: Vec<Vec<vk::CommandBuffer>>,
}

impl CommandPoolPerFrame {
    pub fn new(device: &Device) -> VulkanResult<Self> {
        let pool = CommandPoolBuilder::reset(device).build()?;

        Ok(Self {
            command_pool: pool,
            command_buffers: vec![],
        })
    }

    pub fn allocate_cmd_buffers(
        &mut self,
        device: &Device,
        frame_count: usize,
        pass_count: usize,
    ) -> VulkanResult<&Vec<Vec<vk::CommandBuffer>>> {
        while self.command_buffers.len() < frame_count {
            self.command_buffers.push(vec![]);
        }

        for i in &mut self.command_buffers {
            let dif = pass_count - i.len();
            if dif > 0 {
                let buffers = self
                    .command_pool
                    .create_command_buffers(device, dif as u32)?;
                i.extend(buffers);
            }
        }

        Ok(&self.command_buffers)
    }
}
