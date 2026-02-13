use std::ffi::CStr;
use std::sync::Arc;

use ash::vk;
#[cfg(not(feature = "gpu-allocator"))]
use vk_mem::Allocator;

use super::{Instance, PhysicalDevice, VulkanError, VulkanResult};

/// Logical Device for creation and destroy Vulkan Objects
pub struct Device {
    /// Gpu allocator
    pub(crate) allocator: Arc<Allocator>,
    pub(crate) phys_dev: PhysicalDevice,
    pub(crate) queue_family_props: Vec<vk::QueueFamilyProperties>,
    pub(crate) raw: ash::Device,
}

impl Device {
    pub fn destroy(&mut self) {
        unsafe {
            // ManuallyDrop::drop(&mut self.allocator);
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

        let queue_create_info = vk::DeviceQueueCreateInfo::default()
            .queue_family_index(0)
            .queue_priorities(&[1.0]);

        let p_extenions = self
            .extenions
            .iter()
            .map(|p| p.as_ptr().cast::<i8>())
            .collect::<Vec<_>>();

        let binding = [queue_create_info];

        let create_info = vk::DeviceCreateInfo::default()
            .enabled_extension_names(&p_extenions)
            .queue_create_infos(&binding);

        let device = unsafe {
            self.instance
                .raw
                .create_device(phys_dev.raw, &create_info, None)
                .map_err(VulkanError::Unknown)?
        };

        let queue_prop = unsafe {
            self.instance
                .raw
                .get_physical_device_queue_family_properties(phys_dev.raw)
        };

        let allocator = unsafe {
            let create_info =
                vk_mem::AllocatorCreateInfo::new(&self.instance.raw, &device, phys_dev.raw);
            vk_mem::Allocator::new(create_info)
                .map_err(|_e| VulkanError::Unknown(vk::Result::from_raw(0)))?
        };

        Ok(Device {
            raw: device,
            allocator: Arc::new(allocator),
            phys_dev,
            queue_family_props: queue_prop,
        })
    }
}
