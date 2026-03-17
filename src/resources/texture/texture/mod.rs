#![allow(missing_docs)]

#[derive(Debug)]
pub enum Resolution {
    FullRes,
    HalfRes,
    QuarterRes,
    Custom(u32, u32),
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
