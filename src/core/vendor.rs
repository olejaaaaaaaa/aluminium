/// Known PCI vendor IDs for GPU manufacturers
///
/// List of known IDs referenced from Godot Engine source:
///
/// <https://github.com/godotengine/godot/blob/a3e84cc2af14aa4cffbefd8e13492e567a64e3/servers/rendering/rendering_context_driver.h>
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Vendor {
    Amd,
    Imgtec,
    Apple,
    Nvidia,
    Arm,
    Microsoft,
    Qualcomm,
    Intel,
}
