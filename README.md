## Aluminium 🎮

Lightweight and sometimes unsafe, pure-Rust, graphics library for convenient work with Vulkan Api

[![cargo](https://github.com/olejaaaaaaaa/aluminium/actions/workflows/ci.yaml/badge.svg)](https://github.com/olejaaaaaaaa/aluminium/actions/workflows/ci.yaml)
[![Crates.io](https://img.shields.io/crates/v/aluminium.svg)](https://crates.io/crates/aluminium)
[![Docs](https://docs.rs/aluminium/badge.svg)](https://docs.rs/aluminium)
[![MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://github.com/olejaaaaaaaa/aluminium/blob/main/LICENSE-MIT)
[![Apache 2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://github.com/olejaaaaaaaa/aluminium/blob/main/LICENSE-APACHE)

## Getting Started
You can run the main example
```bash
git clone https://github.com/olejaaaaaaaa/aluminium
cd aluminium
cargo run -p view
```

## Minimal hardware requirments
To support both PC and mobile hardware, only the common subset is used

Extensions

    - VK_KHR_swapchain
    - VK_EXT_descriptor_indexing
    - VK_KHR_driver_properties
    - VK_KHR_synchronization2
    - VK_KHR_get_physical_device_properties2

## Note
Aluminum is focused on data visualization with high enough performance 
It **does not** provide resource loading tools (glTF/OBJ/PNG) and UI display tools(egui/imgui)

## Supported Platforms

| Platform | Status |
|----------|--------|
| Windows  | ✅ ready to use |
| Linux    | 🛠️ in development |
| Android  | 🛠️ in development |
| macOS    | ⚠️ maybe in the future |
| iOS      | ⚠️ maybe in the future |
| Web      | ❌ not supported |

## Credits
This library is heavily inspired by [Kajiya](https://github.com/EmbarkStudios/kajiya). I probably wouldn't have created it if that project didn't exist.

## Contribution
Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions
