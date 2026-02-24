use std::ffi::CStr;

#[derive(Clone, Copy, Debug)]
pub enum Extension {
    /// `VK_KHR_buffer_device_address`
    BufferDeviceAddress,
    /// `VK_KHR_timeline_semaphore`
    TimelineSemaphore,
    /// `VK_KHR_synchronization2`
    Synchronization2,
    /// `VK_KHR_dynamic_rendering`
    DynamicRendering,
    /// `VK_EXT_descriptor_indexing`
    /// 
    /// `VK_KHR_push_descriptor`
    Bindless,
    /// `VK_KHR_acceleration_structure`
    /// 
    /// `VK_KHR_ray_tracing_pipeline`
    RayTracing
} 

impl From<Extension> for &[&'static CStr] {
    fn from(value: Extension) -> Self {
        match value {
            Extension::Bindless => &[
                ash::ext::descriptor_indexing::NAME,
                ash::khr::push_descriptor::NAME
            ],
            Extension::RayTracing => {
                &[
                    ash::khr::acceleration_structure::NAME, 
                    ash::khr::ray_tracing_pipeline::NAME
                ]
            }
            Extension::DynamicRendering => &[ash::khr::dynamic_rendering::NAME],
            Extension::BufferDeviceAddress => &[ash::khr::buffer_device_address::NAME],
            Extension::Synchronization2 => &[ash::khr::synchronization2::NAME],
            Extension::TimelineSemaphore => &[ash::khr::timeline_semaphore::NAME]
        }
    }
}

