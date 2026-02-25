pub trait Resolution {
    fn into_array(&self) -> [u32; 2];
}

impl Resolution for ash::vk::Extent2D {
    fn into_array(&self) -> [u32; 2] {
        [self.width, self.height]
    }
}

pub trait ApiVersion {
    fn version(&self) -> String;
}

impl ApiVersion for u32 {
    fn version(&self) -> String {
        let major = ash::vk::api_version_major(*self);
        let minor = ash::vk::api_version_minor(*self);
        let patch = ash::vk::api_version_patch(*self);
        format!("{}.{}.{}", major, minor, patch)
    }
}
