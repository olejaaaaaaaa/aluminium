// Windows
//
// B8G8R8A8_UNORM	                SRGB_NONLINEAR_KHR	                99.62%
// 0.38% B8G8R8A8_SRGB	                SRGB_NONLINEAR_KHR	                99.53%
// 0.47% A2B10G10R10_UNORM_PACK32	    SRGB_NONLINEAR_KHR	                63.7%
// 36.3%

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
