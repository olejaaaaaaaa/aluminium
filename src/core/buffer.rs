use ash::vk;
use bytemuck::{Pod, Zeroable};
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
    pub fn upload_data<T: Pod + Zeroable>(&mut self, device: &Device, data: &[T]) -> VulkanResult<()> {
        self.vertex_count = data.len() as u32;
        let buffer_size = std::mem::size_of_val(data) as u64;

        let allocation = device.allocator.get_allocation_info(&self.allocation);

        unsafe {
            profiling::scope!("Upload bytes");
            std::ptr::copy_nonoverlapping(data.as_ptr().cast::<u8>(), allocation.mapped_data.cast::<u8>(), buffer_size as usize);
        }

        device
            .allocator
            .flush_allocation(&self.allocation, 0, buffer_size as u64)
            .map_err( 
                VulkanError::Unknown
            )?;

        Ok(())
    }
}

pub struct GpuBufferBuilder<'a> {
    size: Option<u64>,
    usage: Option<vk::BufferUsageFlags>,
    alloc_info: vk_mem::AllocationCreateInfo,
    device: &'a Device,
}

impl<'a> GpuBufferBuilder<'a> {
    pub fn cpu_only(device: &'a Device) -> Self {
        GpuBufferBuilder {
            device,
            usage: None,
            size: None,
            alloc_info: vk_mem::AllocationCreateInfo {
                flags: vk_mem::AllocationCreateFlags::MAPPED | vk_mem::AllocationCreateFlags::HOST_ACCESS_SEQUENTIAL_WRITE,
                usage: vk_mem::MemoryUsage::AutoPreferHost,
                ..Default::default()
            },
        }
    }

    #[allow(dead_code)]
    pub fn gpu_only(device: &'a Device) -> Self {
        GpuBufferBuilder {
            device,
            usage: None,
            size: None,
            alloc_info: vk_mem::AllocationCreateInfo {
                usage: vk_mem::MemoryUsage::AutoPreferDevice,
                ..Default::default()
            },
        }
    }

    pub fn usage(mut self, usage: vk::BufferUsageFlags) -> Self {
        self.usage = Some(usage);
        self
    }

    pub fn size(mut self, size: u64) -> Self {
        self.size = Some(size);
        self
    }

    pub fn build(&self) -> VulkanResult<GpuBuffer> {
        let size = self.size.expect("Missing size");
        let usage = self.usage.expect("Missing usage");

        #[cfg(debug_assertions)]
        {
            if size == 0 {
                panic!("Buffer size cannot be zero");
            }
            if usage.is_empty() {
                panic!("Buffer usage cannot be empty");
            }
        }

        let buffer_info = vk::BufferCreateInfo::default().size(size).usage(usage);

        let (buffer, allocation) = unsafe {
            profiling::scope!("vmaCreateBuffer");
            self.device
                .allocator
                .create_buffer(&buffer_info, &self.alloc_info)
                .map_err(VulkanError::Unknown)
        }?;

        Ok(GpuBuffer {
            raw: buffer,
            vertex_count: 0,
            allocation,
        })
    }
}
