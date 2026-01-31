#[derive(Debug)]
pub enum Resolution {
    Full,
    Custom(u32, u32),
}

#[derive(Debug)]
pub enum SamplerType {
    Linear,
}

#[cfg(target_os = "android")]
mod windows;
#[cfg(target_os = "android")]
pub use windows::TextureFormat;

#[cfg(target_os = "ios")]
mod ios;
#[cfg(target_os = "ios")]
pub use ios::TextureFormat;

#[cfg(target_os = "macos")]
mod mac;
#[cfg(target_os = "macos")]
pub use mac::TextureFormat;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub use linux::TextureFormat;

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub use windows::TextureFormat;

#[derive(Debug)]
pub struct TextureDesc {
    pub resolution: Resolution,
    pub layers: u32,
    pub format: TextureFormat,
    pub sampler: SamplerType,
    pub usage: TextureUsage,
}

impl Default for TextureDesc {
    fn default() -> Self {
        Self {
            resolution: Resolution::Full,
            layers: 1,
            format: TextureFormat::R8g8b8a8Srgb,
            sampler: SamplerType::Linear,
            usage: TextureUsage::Color,
        }
    }
}

#[derive(Debug)]
pub enum TextureUsage {
    Transient,
    Color,
    Depth,
}
