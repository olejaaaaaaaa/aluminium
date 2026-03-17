use std::collections::HashSet;
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
    pub(crate) extensions: HashSet<&'static CStr>,
    pub(crate) features2: vk::PhysicalDeviceFeatures2<'static>,
    pub(crate) driver_props: vk::PhysicalDeviceDriverProperties<'static>,
    pub(crate) queue_family_props: Vec<vk::QueueFamilyProperties>,
    pub(crate) raw: ash::Device,
}

impl Device {
    pub fn vendor(&self) -> vk::DriverId {
        self.driver_props.driver_id
    }

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
    pub fn get_device_extensions(instance: &Instance, phys_dev: &PhysicalDevice) -> VulkanResult<HashSet<&'static CStr>> {
        let mut extensions = HashSet::new();

        let available_extensions = unsafe {
            profiling::scope!("vkEnumerateDeviceExtensionProperties");
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

        debug!("Available device extension: {:#?}", available_extension_names);

        let required_extensions = [
            c"VK_KHR_swapchain",
            c"VK_EXT_descriptor_indexing",
            c"VK_KHR_driver_properties",
            c"VK_KHR_synchronization2",
            c"VK_KHR_timeline_semaphore",
        ];

        for i in required_extensions {
            if !available_extension_names.contains(&i) {
                return Err(VulkanError::LogicalDevice(crate::core::LogicalDeviceError::MissingRequiredExtension(
                    i.to_str().unwrap().to_string(),
                )));
            } else {
                extensions.insert(i);
            }
        }

        let optional_extensions = [
            // Buffer Device Address
            vec![c"VK_KHR_buffer_device_address", c"VK_KHR_device_group"],
        ];

        for i in &optional_extensions {
            let mut is_supported = true;
            for j in i {
                if !available_extension_names.contains(j) {
                    is_supported = false;
                }
            }
            if is_supported {
                extensions.extend(i);
            }
        }

        Ok(extensions)
    }

    fn get_driver_properties(instance: &Instance, phys_dev: &PhysicalDevice) -> vk::PhysicalDeviceDriverProperties<'static> {
        let mut driver_props = vk::PhysicalDeviceDriverProperties::default();
        let mut props2 = vk::PhysicalDeviceProperties2::default().push_next(&mut driver_props);

        unsafe {
            profiling::scope!("vkGetPhysicalDeviceProperties2");
            instance
                .raw
                .get_physical_device_properties2(phys_dev.raw, &mut props2);
        }

        let version = driver_props.conformance_version;
        let conformance_version = format!("0.{}.{}.{}", version.major, version.minor, version.patch);

        log::info!("Conformance Version {:?}", conformance_version);

        driver_props
    }

    fn get_features2(instance: &Instance, phys_dev: &PhysicalDevice) -> vk::PhysicalDeviceFeatures2<'static> {
        let mut features2 = vk::PhysicalDeviceFeatures2::default();
        unsafe {
            profiling::scope!("vkGetPhysicalDeviceFeatures2");
            instance
                .raw
                .get_physical_device_features2(phys_dev.raw, &mut features2);
        }
        features2
    }

    pub fn new(instance: &Instance, phys_dev: &PhysicalDevice) -> VulkanResult<Self> {
        let extensions = Self::get_device_extensions(instance, phys_dev)?;
        let p_extensions = extensions
            .iter()
            .map(|p| p.as_ptr().cast::<i8>())
            .collect::<Vec<_>>();

        let queue_family_prop = unsafe {
            profiling::scope!("vkGetPhysicalDeviceQueueFamilyProperties");
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

        let mut descriptor_indexing = vk::PhysicalDeviceDescriptorIndexingFeatures::default()
            .descriptor_binding_partially_bound(true)
            .descriptor_binding_update_unused_while_pending(true)
            .descriptor_binding_sampled_image_update_after_bind(true)
            .descriptor_binding_storage_image_update_after_bind(true)
            .descriptor_binding_storage_buffer_update_after_bind(true)
            .descriptor_binding_uniform_buffer_update_after_bind(true)
            .runtime_descriptor_array(true);

        let create_info = vk::DeviceCreateInfo::default()
            .queue_create_infos(&queue_infos)
            .enabled_extension_names(&p_extensions)
            .push_next(&mut descriptor_indexing);

        let device = unsafe {
            profiling::scope!("vkCreateDevice");
            instance
                .raw
                .create_device(phys_dev.raw, &create_info, None)
                .map_err(VulkanError::Unknown)?
        };

        debug!("Descriptor indexing feature: {:#?}", descriptor_indexing);
        debug!("Enabled Device Extensions: {:#?}", extensions);

        let allocator = unsafe {
            profiling::scope!("vkCreateVmaAllocator");
            let create_info = vk_mem::AllocatorCreateInfo::new(&instance.raw, &device, phys_dev.raw);
            vk_mem::Allocator::new(create_info).map_err(|_e| VulkanError::Unknown(vk::Result::from_raw(0)))
        }?;

        let features2 = Self::get_features2(instance, phys_dev);
        let driver_props = Self::get_driver_properties(instance, phys_dev);

        Ok(Device {
            raw: device,
            extensions,
            driver_props,
            features2,
            allocator: ManuallyDrop::new(allocator),
            queue_family_props: queue_family_prop,
        })
    }
}
