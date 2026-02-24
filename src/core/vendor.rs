/// Known PCI vendor IDs for GPU manufacturers
/// 
/// List of known IDs referenced from Godot Engine source:
/// 
/// <https://github.com/godotengine/godot/blob/a3e84cc2af14aa4cffbefd8e13492e567a64e3/servers/rendering/rendering_context_driver.h>
#[derive(Clone, Copy, Debug)]
pub enum Vendor {
    Unknown = 0x0,
    Amd = 0x1002,
    Imgtec = 0x1010,
    Apple = 0x106B,
    Nvidia = 0x10DE,
    Arm = 0x13B5,
    Microsoft = 0x1414,
    Qualcomm = 0x5143,
    Intel = 0x8086,
}

impl From<u32> for Vendor {
    fn from(value: u32) -> Self {
        match value {
            0x1002 => Vendor::Amd,
            0x1010 => Vendor::Imgtec,
            0x106B => Vendor::Apple,
            0x10DE => Vendor::Nvidia,
            0x13B5 => Vendor::Arm,
            0x1414 => Vendor::Microsoft,
            0x5143 => Vendor::Qualcomm,
            0x8086 => Vendor::Intel,
            _ => Vendor::Unknown,
        }
    }
}