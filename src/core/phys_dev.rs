use ash::vk;
use log::info;

use crate::core::{Instance, Vendor, VulkanError, VulkanResult};

const GPU_PRIORITY: [vk::PhysicalDeviceType; 5] = [
    vk::PhysicalDeviceType::DISCRETE_GPU,
    vk::PhysicalDeviceType::INTEGRATED_GPU,
    vk::PhysicalDeviceType::VIRTUAL_GPU,
    vk::PhysicalDeviceType::CPU,
    vk::PhysicalDeviceType::OTHER,
];

pub struct PhysicalDevice {
    pub(crate) raw: vk::PhysicalDevice,
    pub(crate) prop: vk::PhysicalDeviceProperties,
}

impl PhysicalDevice {
    pub fn new(instance: &Instance) -> VulkanResult<Self> {

        let phys_devs = unsafe { instance.raw.enumerate_physical_devices() .map_err(VulkanError::Unknown) }?;

        for ty in GPU_PRIORITY {
            for dev in &phys_devs {
                
                let prop = unsafe { instance.raw.get_physical_device_properties(*dev) };
                
                if prop.device_type == ty {

                    let limits = prop.limits;
                    info!("GPU Type: {:?}", prop.device_type);
                    info!("Vendor: {:?}", Into::<Vendor>::into(prop.vendor_id));
                    info!("Max Storage Buffers: {}", limits.max_descriptor_set_storage_buffers);
                    info!("Max Push Constants: {}", limits.max_push_constants_size);
                    info!("Max Indirect Draw: {}", limits.max_draw_indirect_count);
                    info!("Max Dynamic Uniforms: {}", limits.max_descriptor_set_uniform_buffers_dynamic);

                    return Ok(Self { raw: *dev, prop });
                }
            }
        }

        Err(VulkanError::Unknown(vk::Result::from_raw(0)))
    }

    pub(crate) fn limits(&self) -> &vk::PhysicalDeviceLimits {
        &self.prop.limits
    }
}

impl std::ops::Deref for PhysicalDevice {
    type Target = ash::vk::PhysicalDevice;
    fn deref(&self) -> &Self::Target {
        &self.raw
    }
}
