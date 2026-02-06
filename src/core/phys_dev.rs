use ash::vk;

pub struct PhysicalDevice {
    pub(crate) raw: vk::PhysicalDevice,
    #[allow(dead_code)]
    pub(crate) prop: vk::PhysicalDeviceProperties,
}

impl PhysicalDevice {
    #[allow(dead_code)]
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
