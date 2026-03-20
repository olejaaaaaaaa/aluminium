# Aluminium 🎮

Lightweight and sometimes unsafe, pure-Rust, graphics library for convenient work with Vulkan Api

![GitHub](https://img.shields.io/github/license/olejaaaaaaaa/aluminium)
[![cargo](https://github.com/olejaaaaaaaa/aluminium/actions/workflows/ci.yaml/badge.svg)](https://github.com/olejaaaaaaaa/aluminium/actions/workflows/ci.yaml)
[![Hits-of-Code](https://hitsofcode.com/github/olejaaaaaaaa/aluminium)](https://hitsofcode.com/github/olejaaaaaaaa/aluminium/view)

# Warning
This library is currently unstable and its API is subject to frequent changes

# Getting Started
You can run the main example
```bash
git clone https://github.com/olejaaaaaaaa/aluminium
cd aluminium
cargo run -p simple
```
# Usage
Aluminium is focused on data visualization with reasonably high performance 
It does **not** provide asset loaders (glTF / OBJ / PNG) — bring your own

# Minimal required extensions
To support both PC and mobile hardware, only the common subset is used

    - VK_KHR_swapchain
    - VK_EXT_descriptor_indexing
    - VK_KHR_driver_properties
    - VK_KHR_synchronization2
    - VK_KHR_timeline_semaphore
    - VK_KHR_get_physical_device_properties2

# Features:
* VMA Allocator integration
* Simple FrameGraph

## Supported Platforms

| Platform | Status |
|----------|--------|
| Windows  | ✅ |
| Linux    | 🔜 planned |
| Android  | 🔜 planned |
| macOS    | ⚠️ Unsupported | May work via MoltenVK, no guarantees |
| iOS      | ⚠️ Unsupported | Same as macOS |

# Credits
This library is heavily inspired by [Kajiya](https://github.com/EmbarkStudios/kajiya). I probably wouldn't have created it if that project didn't exist.
