#![allow(missing_docs)]

use std::error::Error;
use std::time::Instant;

use aluminium::types::PbrVertex;
use aluminium::{Get, GetMut, Handle, LoadOp, Location, RasterPass, RasterPipeline, RasterPipelineDesc, RenderTarget, Res, Resolution, Scissor, ShaderType, StorageBuffer, StorageBufferDesc, StoreOp, Texture, TextureDesc, TextureFormat, TransientTexture, VertexInput, Viewport, WorldRenderer};
use bytemuck::{Pod, Zeroable};
use tracing_subscriber::filter::LevelFilter;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};
use winit::*;

use crate::{GltfModel, UiRenderer, load_gltf};


#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct Test {
    time: f32
}

pub struct View {
    global_time: std::time::Instant,
    model: GltfModel,
    gbuffer_pipeline: Res<RasterPipeline>,
    ui: UiRenderer,
    world: WorldRenderer,
}

impl View {

    pub fn new(window: &Window) -> Self {

        let world = WorldRenderer::new(window).expect("Error create world renderer");
        let ui = UiRenderer::new(&world, window).expect("Error create UI renderer");

        let gbuffer_pipeline = world
            .create::<RasterPipeline>(
                RasterPipelineDesc::new()
                    .vertex_shader("./shaders/spv/raster_vs.spv")
                    .fragment_shader("./shaders/spv/raster_ps.spv")
                    .vertex_input::<PbrVertex>()
                    .depth_test(true)
                    .dynamic_scissors(true)
                    .dynamic_viewport(true)
            )
            .expect("Error create pipeline");

        let model = load_gltf(&world, "./examples/view/assets/flighthelmet/scene.gltf").expect("Error load gltf model");

        Self {
            global_time: Instant::now(),
            model,
            gbuffer_pipeline,
            ui,
            world
        }
    }

    pub fn handle_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
       match event {
            WindowEvent::Resized(size) => {
                let (width, height) = (size.width, size.height);
                let world = &mut self.world;
                world.resize(width, height).expect("Error resize window");
            },
            WindowEvent::RedrawRequested => {

                let gbuffer_pipeline = &self.gbuffer_pipeline;
                let time_sec = self.global_time.elapsed().as_secs_f32();
                
                let model = &self.model;
                let ssbo = model.ssbo.as_ref().unwrap();

                let _ = self.world.draw_frame(move |frame| {

                    #[derive(Clone, Copy)]
                    pub struct GBuffer {
                        albedo: Handle<TransientTexture>,
                        depth: Handle<TransientTexture>,
                    }
   
                    let gbuffer: GBuffer = frame.add_pass(
                        RasterPass::new("GBuffer Pass")
                            .setup(|builder| {

                                builder.read_storage_buffer(ssbo, Location { set: 2, binding: 0 });

                                let albedo: Handle<TransientTexture> = builder.create_texture(
                                    "albedo", 
                                    TextureFormat::Color, 
                                    Resolution::FullRes
                                );

                                let depth: Handle<TransientTexture> = builder.create_texture(
                                    "depth",
                                    TextureFormat::Depth,
                                    Resolution::FullRes,
                                );

                                let albedo: Handle<TransientTexture> = builder.write_color(albedo, LoadOp::Clear, StoreOp::Store);
                                let depth: Handle<TransientTexture> = builder.write_depth(depth, LoadOp::Clear, StoreOp::Store);

                                GBuffer {
                                    albedo,
                                    depth
                                }
                            })
                            .execute(move |ctx| unsafe {
                                ctx.bind_pipeline(gbuffer_pipeline);
                                ctx.set_scissor(Scissor::FullRes);
                                ctx.set_viewport(Viewport::FullRes);
                                for (index, (mesh, material)) in model.meshes.iter().enumerate() {
                                    ctx.push_constants([
                                        0.0, 15.0 * time_sec.sin().abs(), 15.0 * time_sec.cos().abs(),
                                        index as f32,
                                    ]);
                                    let textures = vec![
                                        model.textures.get(material.diffuse_map as usize).unwrap_or(&model.textures[0]),
                                        model.textures.get(material.metallic_roughness_map as usize).unwrap_or(&model.textures[0]),
                                        model.textures.get(material.occlusion_map as usize).unwrap_or(&model.textures[0]),
                                        model.textures.get(material.normal_map as usize).unwrap_or(&model.textures[0])
                                    ];
                                    ctx.bind_texture(&textures);
                                    ctx.draw_indexed(&mesh.vertex, &mesh.index);
                                }
                            }),
                    );

                    // let () = frame.add_pass(
                    //     RasterPass::new("Final Pass")
                    //         .setup(move |builder| {

                    //             builder.read_texture(gbuffer.albedo, Location { set: 1, binding: 0 });

                    //             let backbuffer = builder.backbuffer();
                    //             let _ = builder.write_color(backbuffer, LoadOp::Clear, StoreOp::DontCare);
                    //             let _ = builder.write_depth(gbuffer.depth, LoadOp::Clear, StoreOp::DontCare);

                    //         })
                    //         .execute(move |ctx| unsafe {
                    //             ctx.bind_pipeline(pipeline);
                    //             ctx.set_scissor(Scissor::FullRes);
                    //             ctx.set_viewport(Viewport::FullRes);
                    //             ctx.draw_fullscreen();
                    //         })
                    // );
                });
            },
            _ => {

            }
       }
    }
}