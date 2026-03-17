use ash::vk;
use bytemuck::{Pod, Zeroable};
use vk_mem::{Alloc, Allocation};

use crate::core::Device;
use crate::{VulkanError, VulkanResult};

pub struct RingBuffer {
    pub raw: vk::Buffer,
    allocation: Allocation,
    capacity: u64,
    frame_count: usize,
    current: usize,
}

impl RingBuffer {
    pub fn new(device: &Device, capacity: u64, frame_count: usize, usage: vk::BufferUsageFlags) -> VulkanResult<Self> {
        let total_size = capacity * frame_count as u64;

        let (raw, allocation) = unsafe {
            device
                .allocator
                .create_buffer(
                    &vk::BufferCreateInfo::default()
                        .size(total_size)
                        .usage(usage),
                    &vk_mem::AllocationCreateInfo {
                        flags: vk_mem::AllocationCreateFlags::MAPPED | vk_mem::AllocationCreateFlags::HOST_ACCESS_SEQUENTIAL_WRITE,
                        usage: vk_mem::MemoryUsage::AutoPreferHost,
                        ..Default::default()
                    },
                )
                .map_err(VulkanError::Unknown)?
        };

        Ok(Self {
            raw,
            allocation,
            capacity,
            frame_count,
            current: 0,
        })
    }

    pub unsafe fn write<T: Pod + Zeroable>(&self, device: &Device, data: &[T]) -> VulkanResult<u64> {
        let buffer_size = std::mem::size_of_val(data) as u64;
        let offset = self.current_offset();

        let allocation = device.allocator.get_allocation_info(&self.allocation);

        unsafe {
            std::ptr::copy_nonoverlapping(
                data.as_ptr().cast::<u8>(),
                allocation.mapped_data.cast::<u8>().add(offset as usize),
                buffer_size as usize,
            );
        }

        device
            .allocator
            .flush_allocation(&self.allocation, 0, buffer_size as u64)
            .map_err(|e| VulkanError::Unknown(e))?;

        Ok(offset)
    }

    pub fn advance(&mut self) {
        self.current = (self.current + 1) % self.frame_count;
    }

    pub fn current_offset(&self) -> u64 {
        self.capacity * self.current as u64
    }
}
