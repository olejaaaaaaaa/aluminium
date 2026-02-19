use std::ffi::CStr;
use std::mem::ManuallyDrop;

use ash::vk;
#[cfg(not(feature = "gpu-allocator"))]
use vk_mem::Allocator;

use super::{Instance, PhysicalDevice, VulkanError, VulkanResult};

/// Logical Device for creation and destroy Vulkan Objects
pub struct Device {
    /// Gpu allocator
    pub(crate) allocator: ManuallyDrop<Allocator>,
    pub(crate) phys_dev: PhysicalDevice,
    pub(crate) queue_family_props: Vec<vk::QueueFamilyProperties>,
    pub(crate) raw: ash::Device,
}

impl Device {
    pub fn destroy(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.allocator);
            self.raw.destroy_device(None);
        }
    }
}

impl std::ops::Deref for Device {
    type Target = ash::Device;
    fn deref(&self) -> &ash::Device {
        &self.raw
    }
}

pub struct DeviceBuilder<'a> {
    phys_dev: PhysicalDevice,
    instance: &'a Instance,
    extenions: Vec<&'static CStr>,
}

impl<'a> DeviceBuilder<'a> {
    pub fn default(instance: &'a Instance, phys_dev: PhysicalDevice) -> Self {
        DeviceBuilder {
            instance,
            phys_dev,
            extenions: vec![c"VK_KHR_swapchain"],
        }
    }

    #[allow(dead_code)]
    pub fn with_extenions<S: Iterator<Item = &'static CStr>>(mut self, extensions: S) -> Self {
        let extensions = extensions.into_iter().collect::<Vec<_>>();

        self.extenions.extend(extensions);
        self
    }

    pub fn build(self) -> VulkanResult<Device> {
        let phys_dev = self.phys_dev;

        let p_extenions = self
            .extenions
            .iter()
            .map(|p| p.as_ptr().cast::<i8>())
            .collect::<Vec<_>>();

        let queue_family_prop = unsafe {
            self.instance
                .raw
                .get_physical_device_queue_family_properties(phys_dev.raw)
        };

        let mut priorities: Vec<Vec<f32>> = vec![];

        for i in &queue_family_prop {
            priorities.push(
                (1..i.queue_count + 1)
                    .map(|ndx| 1.0 / (ndx as f32))
                    .collect::<Vec<f32>>(),
            );
        }

        let mut queue_infos = vec![];

        for (index, _) in queue_family_prop.iter().enumerate() {
            let queue_info = vk::DeviceQueueCreateInfo::default()
                .queue_family_index(index as u32)
                .queue_priorities(&priorities[index]);

            queue_infos.push(queue_info);
        }

        let create_info = vk::DeviceCreateInfo::default()
            .queue_create_infos(&queue_infos)
            .enabled_extension_names(&p_extenions);

        let device = unsafe {
            self.instance
                .raw
                .create_device(phys_dev.raw, &create_info, None)
                .map_err(VulkanError::Unknown)?
        };

        let allocator = unsafe {
            let create_info =
                vk_mem::AllocatorCreateInfo::new(&self.instance.raw, &device, phys_dev.raw);
            vk_mem::Allocator::new(create_info)
                .map_err(|_e| VulkanError::Unknown(vk::Result::from_raw(0)))?
        };

        Ok(Device {
            raw: device,
            allocator: ManuallyDrop::new(allocator),
            phys_dev,
            queue_family_props: queue_family_prop,
        })
    }
}
