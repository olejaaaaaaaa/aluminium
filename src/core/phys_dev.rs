use ash::vk;

pub struct PhysicalDevice {
    pub(crate) raw: vk::PhysicalDevice,
    pub(crate) prop: vk::PhysicalDeviceProperties,
}

impl PhysicalDevice {
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
