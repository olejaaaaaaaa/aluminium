#![allow(missing_docs)]

use std::error::Error;

use aluminium::types::Vertex;
use aluminium::{
    Material, MaterialHandle, MeshHandle, PresentPassBuilder, RasterPassBuilder, Renderable, RenderableHandle, Resolution, SamplerType, ShaderStage, ShaderType, TextureDesc, TextureFormat, TextureUsage, Transform, TransformHandle, UniformBinding, UniformValue, VertexInput, VulkanError, VulkanResult, WorldRenderer
};
use bytemuck::{Pod, Zeroable};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};
use winit::*;

struct GameObject {
    transform: TransformHandle,
    mesh: MeshHandle,
    color: MaterialHandle,
    renderable: RenderableHandle
}

#[derive(Default)]
struct App {
    game_object: Option<GameObject>,
    world: Option<WorldRenderer>,
    window: Option<winit::window::Window>,
}

impl ApplicationHandler for App {
    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            },
            WindowEvent::Resized(size) => {
                let (width, height) = (size.width, size.height);
                let world = self.world.as_mut().unwrap();
                world.resize(width, height).expect("Error resize window");
            },
            WindowEvent::RedrawRequested => {
                let window = self.window.as_ref().unwrap();
                window.pre_present_notify();

                let world = self.world.as_mut().unwrap();
                let game_object = self.game_object.as_ref().unwrap();

                world.with_assets_mut(|assets| {
                    let color = assets.get_mut_material(game_object.color).unwrap();
                    let value = color.get_mut::<&str, UniformValue>("Time").unwrap();
                    match value {
                        UniformValue::Float(f) => {
                            *f += 0.01;  
                        },
                        _ => eprintln!("Not a float"),
                    }
                    Ok(())
                }).unwrap();

                let _ = world.draw_frame(|graph| {
                    graph.add_pass(
                        PresentPassBuilder::new()
                            .use_bindless()
                            .dynamic_scissors()
                            .dynamic_viewport()
                            .vertex("./shaders/spv/raster_vs-hlsl.spv")
                            .fragment("./shaders/spv/raster_ps-hlsl.spv")
                            .vertex_attributes(&[
                                // Pos
                                ShaderType::Float3,
                                // Color
                                ShaderType::Float3,
                                // Time
                                ShaderType::Float3,
                            ])
                            .uniforms(&[
                                // Time
                                UniformBinding {
                                    set: 1,
                                    bind: 0,
                                    ty: ShaderType::Float,
                                    stage: ShaderStage::Vertex,
                                },
                            ])
                            .custom(|ctx, renderables| unsafe {
                                ctx.bind_bindless();
                                ctx.set_scissor(None);
                                ctx.set_viewport(None);
                                ctx.bind_pipeline();
                                for i in renderables {
                                    ctx.bind_materials(i);
                                    ctx.draw_mesh(i);
                                }
                            }),
                    );
                });
            },
            _ => (),
        }
    }

    fn about_to_wait(&mut self, _: &ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }

    fn resumed(&mut self, event_loop: &event_loop::ActiveEventLoop) {
        let window_attributes = Window::default_attributes()
            .with_title("Game")
            .with_inner_size(winit::dpi::LogicalSize::new(640.0, 480.0));

        let window = event_loop
            .create_window(window_attributes)
            .expect("Error create window");

        let world = WorldRenderer::new(&window).expect("Error create world renderer");

        let game_object = world.with_assets_mut(|assets| {

            #[repr(C)]
            #[derive(Pod, Zeroable, Copy, Clone)]
            pub struct CustomVertex {
                pos: [f32; 3],
                color: [f32; 3],
                time: [f32; 3]
            }

            let triangle_mesh = vec![
                CustomVertex {
                    pos: [0.0, 0.5, 0.0],
                    color: [0.0, 1.0, 0.0],
                    time: [1.0, 0.0, 0.0]
                },
                CustomVertex {
                    pos: [-0.5, -0.5, 0.0],
                    color: [0.0, 0.0, 1.0],
                    time: [0.0, 1.0, 0.0]
                },
                CustomVertex {
                    pos: [0.5, -0.5, 0.0],
                    color: [0.0, 0.0, 1.0],
                    time: [0.0, 0.0, 1.0]
                },
            ];

            let color = assets
                .create_material(Material::new(1).set_value("Time", 0.0))?;

            let mesh = assets.create_mesh(&triangle_mesh, None)?;
            let transform = assets.create_transform(Transform::identity())?;
            let renderable = assets.create_renderable(Renderable::new(mesh, &[color], transform));

            Ok(GameObject {
                color,
                mesh,
                transform,
                renderable
            })
        }).unwrap();

        self.game_object = Some(game_object);
        self.world = Some(world);
        self.window = Some(window);
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    unsafe {
        std::env::set_var("RUST_LOG", "Debug");
    }

    env_logger::builder().init();

    let event_loop = EventLoop::new()?;
    event_loop.run_app(&mut App::default())?;

    Ok(())
}
