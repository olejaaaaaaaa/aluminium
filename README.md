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

## Example - Triangle

```rust
// The structure through which all interaction will take place
let world = WorldRenderer::new(&window);

// The Rasterization Pipeline is used to render an image to the 
// screen or to an offscreen texture and cannot be changed after creation
let pipeline: Res<RasterPipeline> = world
    .create::<RasterPipeline>(
        RasterPipelineDesc::new()
            // You can pass either the path to the spv byte code or directly pass a slice of bytes
            .vertex_shader("./shaders/spv/raster_vs.spv")
            // The shader must have the correct header for this type of pipeline
            // See the usage example in the file ./shaders/raster_vs.hlsl and ./shaders/raster_ps.hlsl
            .fragment_shader("./shaders/spv/raster_ps.spv")
            // The format of vertices that this pipeline can work with
            .vertex_input::<Vertex>()
            // Enable depth testing to properly display the 3D mesh
            // If this flag is enabled, you should use a depth texture when rendering in the setup phase
            .depth_test(true)
            // We can dynamically crop the image if we want
            // If the flag is enabled, you need to configure this parameter in the execute phase
            .dynamic_scissors(true)
            // Auto resize
            // If the flag is enabled, you need to configure this parameter in the execute phase
            .dynamic_viewport(true)
    )
    .expect("Error create pipeline");

let vertices = vec![
    Vertex { pos: [0.0, 1.0, 0.0], color: [1.0, 0.0, 0.0] },
    Vertex { pos: [-1.0, -1.0, 0.0], color: [0.0, 1.0, 0.0] },
    Vertex { pos: [0.0, 1.0, -1.0], color: [0.0, 0.0, 1.0] }
];

// Create new Buffer for all vertices
// May throw an error if there is not enough memory for allocation
let vertex_buffer: Res<VertexBuffer> = world.create::<VertexBuffer>(VertexBufferDesc::new(&vertices)).expect("Error create Vertex Buffer");

// Indices for vertices
// May throw an error if there is not enough memory for allocation
let index_buffer: Res<IndexBuffer> = world.create::<IndexBuffer>(IndexBufferDesc::new(&vec![0, 1, 2u32])).expect("Error create Index Buffer");

// The main closure in which all rendering passes will be defined
// We can handle the case when the frame was not rendered for some reason
// (lack of memory, driver error, device loss), but in most cases I will just ignore it
let _ = world.draw_frame(move |frame| {
    // An explicit indication of the output value is required even if we do not return anything from the setup phase.
    let () = frame.add_pass(
        RasterPass::new("Simple Pass")
            .setup(|builder| {

                // Getting a texture for display on the screen
                let back: Handle<TransientTexture> = builder.backbuffer();

                // Create a Depth texture for depth testing
                let depth: Handle<TransientTexture> = builder.create_texture(
                    // Name for debug and profiling
                    "depth",
                    // Standart Format for Depth image without Stencil
                    TextureFormat::Depth,
                    // Full Resolution
                    Resolution::FullRes,
                );

                // Writing to a BackBuffer texture
                let _ = builder.write_color(back, LoadOp::Clear, StoreOp::Store);
                // Writing to a Depth texture
                let _ = builder.write_depth(depth, LoadOp::Clear, StoreOp::DontCare);
            })
            .execute(move |ctx| unsafe {
                // Incorrect use will cause the driver to crash
                // The order of execution is critically important
                // There are minimal checks for the correctness of the transmitted data
                ctx.bind_pipeline(pipeline);
                // The Push Constants function can be omitted
                // Some amount of data can be passed through push_constants instead of binding uniforms
                ctx.push_constants([1.0, 0.2f32]);
                // The pipeline must be created with the dynamic_scissors flag
                ctx.set_scissor(Scissor::FullRes);
                // The pipeline must be created with the dynamic_viewport flag
                // viewport for current window size
                ctx.set_viewport(Viewport::FullRes);
                // Draw mesh
                ctx.draw_indexed(&vertex_buffer, &index_buffer);
            }),
    );
});
```

## Shaders
Any shader language works as long as it creates a valid SPIR-V and
conforms to the pipeline layout. I use GLSL and HLSL mostly

## Minimal hardware requirments
To support both PC and mobile hardware, only the common subset is used

I use Vulkan API 1.1+/1.2 version

Required extensions

    - VK_KHR_swapchain
    - VK_EXT_descriptor_indexing
    - VK_KHR_driver_properties
    - VK_KHR_get_physical_device_properties2
    - VK_KHR_imageless_framebuffer
    - VK_KHR_buffer_device_address
    - VK_KHR_timeline_semaphore
    - VK_KHR_device_group


Required formats

    D32_SFLOAT          (SAMPLED/DEPTH_STENCIL)
    R8G8B8A8_SRGB       (SAMPLED/COLOR_ATTACHEMENT)
    R16G16B16A16_SFLOAT (SAMPLED/COLOR_ATTACHMENT)

Other extensions and formats are optional and may not be supported on the target device

## Note
Aluminum is a **data visualization** engine optimized for rendering performance

It intentionally excludes:

    - Asset loaders (glTF, OBJ, PNG, etc.)
    - Ui tools (egui, imgui, etc.)

## Known issues
Not all resources are cleared correctly

I'm balancing the speed of adding new features and correctness

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
