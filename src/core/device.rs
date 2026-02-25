use std::ffi::CStr;
use std::mem::ManuallyDrop;

use ash::vk;
use log::{debug, info};
#[cfg(not(feature = "gpu-allocator"))]
use vk_mem::Allocator;

use super::{Instance, PhysicalDevice, VulkanError, VulkanResult};

/// Logical Device for creation and destroy Vulkan Objects
pub struct Device {
    /// Gpu allocator
    pub(crate) allocator: ManuallyDrop<Allocator>,
    pub(crate) extensions: Vec<&'static CStr>,
    pub(crate) queue_family_props: Vec<vk::QueueFamilyProperties>,
    pub(crate) raw: ash::Device,
}

impl Device {
    pub fn check_extensions(&self, extensions: &[&'static CStr]) -> bool {
        for i in extensions {
            if !self.extensions.contains(i) {
                return false;
            }
        }
        true
    }

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

impl Device {
    pub fn new(instance: &Instance, phys_dev: &PhysicalDevice) -> VulkanResult<Self> {
        let available_extensions = unsafe {
            instance
                .raw
                .enumerate_device_extension_properties(phys_dev.raw)
                .map_err(VulkanError::Unknown)
        }?;

        let mut available_extension_names = vec![];

        for i in &available_extensions {
            if let Ok(name) = i.extension_name_as_c_str() {
                available_extension_names.push(name);
            }
        }

        debug!(
            "Available device extension: {:#?}",
            available_extension_names
        );

        if !available_extension_names.contains(&c"VK_KHR_swapchain") {
            panic!("Not supported VK_KHR_swapchain")
        }

        let mut extensions = vec![c"VK_KHR_swapchain"];

        #[cfg(target_os = "windows")]
        {
            let mut is_bindless_supported = false;
            if available_extension_names.contains(&c"VK_KHR_push_descriptor")
                && available_extension_names.contains(&c"VK_KHR_maintenance3")
                && available_extension_names.contains(&c"VK_EXT_descriptor_indexing")
            {
                // Bindless
                extensions.push(c"VK_KHR_maintenance3");
                extensions.push(c"VK_EXT_descriptor_indexing");
                extensions.push(c"VK_KHR_push_descriptor");
                is_bindless_supported = true;
            }

            info!("Bindless supported: {}", is_bindless_supported);

            let mut is_sync2_supported = false;
            if available_extension_names.contains(&c"VK_KHR_synchronization2") {
                // Sync2
                extensions.push(c"VK_KHR_synchronization2");
                is_sync2_supported = true;
            }

            info!("Sync2 supported: {}", is_sync2_supported);

            let mut is_timeline_semaphore_supported = false;
            if available_extension_names.contains(&c"VK_KHR_timeline_semaphore") {
                // Timeline
                extensions.push(c"VK_KHR_timeline_semaphore");
                is_timeline_semaphore_supported = true;
            }

            info!(
                "Timeline Semaphore supported: {}",
                is_timeline_semaphore_supported
            );

            let mut is_dynamic_rendering_supported = false;
            if available_extension_names.contains(&c"VK_KHR_dynamic_rendering")
                && available_extension_names.contains(&c"VK_KHR_create_renderpass2")
                && available_extension_names.contains(&c"VK_KHR_multiview")
                && available_extension_names.contains(&c"VK_KHR_maintenance2")
                && available_extension_names.contains(&c"VK_KHR_depth_stencil_resolve")
            {
                // Dynamic Rendering
                extensions.push(c"VK_KHR_multiview");
                extensions.push(c"VK_KHR_maintenance2");
                extensions.push(c"VK_KHR_create_renderpass2");
                extensions.push(c"VK_KHR_depth_stencil_resolve");
                extensions.push(c"VK_KHR_dynamic_rendering");
                is_dynamic_rendering_supported = true;
            }

            info!(
                "Dynamic Rendering supported: {}",
                is_dynamic_rendering_supported
            );

            let mut is_buffer_device_address_supported = false;
            if available_extension_names.contains(&c"VK_KHR_buffer_device_address")
                && instance
                    .extensions
                    .contains(&c"VK_KHR_device_group_creation")
                && available_extension_names.contains(&c"VK_KHR_device_group")
            {
                // Buffer Device Address
                extensions.push(c"VK_KHR_device_group");
                extensions.push(c"VK_KHR_buffer_device_address");
                is_buffer_device_address_supported = true;
            }

            info!(
                "Buffer Device Address supported: {}",
                is_buffer_device_address_supported
            );
        }

        let queue_family_prop = unsafe {
            instance
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

        let p_extensions = extensions
            .iter()
            .map(|p| p.as_ptr().cast::<i8>())
            .collect::<Vec<_>>();

        let create_info = vk::DeviceCreateInfo::default()
            .queue_create_infos(&queue_infos)
            .enabled_extension_names(&p_extensions);

        let device = unsafe {
            instance
                .raw
                .create_device(phys_dev.raw, &create_info, None)
                .map_err(VulkanError::Unknown)?
        };

        debug!("Enabled Device Extensions: {:#?}", extensions);

        let allocator = unsafe {
            let create_info =
                vk_mem::AllocatorCreateInfo::new(&instance.raw, &device, phys_dev.raw);
            vk_mem::Allocator::new(create_info)
                .map_err(|_e| VulkanError::Unknown(vk::Result::from_raw(0)))
        }?;

        Ok(Device {
            raw: device,
            extensions,
            allocator: ManuallyDrop::new(allocator),
            queue_family_props: queue_family_prop,
        })
    }
}
