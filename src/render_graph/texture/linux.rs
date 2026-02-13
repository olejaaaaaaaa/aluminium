// Linux
//
// B8G8R8A8_UNORM	                SRGB_NONLINEAR_KHR	                87.76%
// 12.24% B8G8R8A8_SRGB	                SRGB_NONLINEAR_KHR
// 87.32%	12.68% A2R10G10B10_UNORM_PACK32	    SRGB_NONLINEAR_KHR
// 28.06%	71.94%

// pub enum TextureFormat {
//     D32Sfloat,
//     B8G8R8A8_UNORM,
//     B8G8R8A8_SRGB,
//     A2R10G10B10_UNORM_PACK32,
// }

/// Formats for Texture
#[derive(Debug)]
pub enum TextureFormat {
    /// For Depth
    D32Sfloat,
    /// For Image
    R8g8b8a8Srgb,
    /// For Image
    R8g8b8a8Unorm,
}
