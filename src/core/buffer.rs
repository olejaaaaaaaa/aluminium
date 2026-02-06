use ash::vk;
use bytemuck::{Pod, Zeroable};
use puffin::profile_scope;
#[cfg(all(feature = "vma", not(feature = "gpu-allocator")))]
use vk_mem::Alloc;
#[cfg(all(feature = "vma", not(feature = "gpu-allocator")))]
use vk_mem::Allocation;

#[cfg(feature = "gpu-allocator")]
compile_error!("Not supported at the moment");

#[cfg(not(any(feature = "vma", feature = "gpu-allocator")))]
compile_error!("At least one allocator feature must be enabled: 'vma' or 'gpu-allocator'");

use super::{Device, VulkanError, VulkanResult};

pub struct GpuBuffer {
    pub raw: vk::Buffer,
    pub vertex_count: u32,
    pub allocation: Allocation,
}

impl GpuBuffer {
    /// Copy slice as raw bytes into [`vk::Buffer`]
    pub fn upload_data<T: Pod + Zeroable>(
        &mut self,
        device: &Device,
        data: &[T],
    ) -> VulkanResult<()> {
        self.vertex_count = data.len() as u32;
        let buffer_size = (data.len() * std::mem::size_of::<T>()) as u64;

        profile_scope!("Upload bytes");

        let allocation = device.allocator.get_allocation_info(&self.allocation);

        unsafe {
            std::ptr::copy_nonoverlapping(
                data.as_ptr() as *const u8,
                allocation.mapped_data as *mut u8,
                buffer_size as usize,
            );
        }

        device
            .allocator
            .flush_allocation(&self.allocation, 0, buffer_size as u64)
            .map_err(|e| {
                use crate::core::errors::VulkanError;
                VulkanError::Unknown(e)
            })?;

        Ok(())
    }
}

pub struct GpuBufferBuilder<'a> {
    buffer_info: vk::BufferCreateInfo<'static>,
    alloc_info: vk_mem::AllocationCreateInfo,
    device: &'a Device,
}

impl<'a> GpuBufferBuilder<'a> {
    pub fn cpu_only(device: &'a Device) -> Self {
        GpuBufferBuilder {
            device,
            buffer_info: vk::BufferCreateInfo::default(),
            alloc_info: vk_mem::AllocationCreateInfo {
                flags: vk_mem::AllocationCreateFlags::MAPPED
                    | vk_mem::AllocationCreateFlags::HOST_ACCESS_SEQUENTIAL_WRITE,
                usage: vk_mem::MemoryUsage::AutoPreferHost,
                ..Default::default()
            },
        }
    }

    #[allow(dead_code)]
    pub fn gpu_only(device: &'a Device) -> Self {
        GpuBufferBuilder {
            device,
            buffer_info: vk::BufferCreateInfo::default(),
            alloc_info: vk_mem::AllocationCreateInfo {
                usage: vk_mem::MemoryUsage::AutoPreferDevice,
                ..Default::default()
            },
        }
    }

    pub fn usage(mut self, usage: vk::BufferUsageFlags) -> Self {
        self.buffer_info = self.buffer_info.usage(usage);
        self
    }

    pub fn size(mut self, size: u64) -> Self {
        self.buffer_info = self.buffer_info.size(size);
        self
    }

    pub fn build(&self) -> VulkanResult<GpuBuffer> {
        profile_scope!("GpuBuffer");

        let (buffer, allocation) = unsafe {
            self.device
                .allocator
                .create_buffer(&self.buffer_info, &self.alloc_info)
                .map_err(|e| VulkanError::Unknown(e))
        }?;

        Ok(GpuBuffer {
            raw: buffer,
            vertex_count: 0,
            allocation,
        })
    }
}
