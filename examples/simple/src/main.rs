#![allow(missing_docs)]

use std::error::Error;

use aluminium::types::Vertex;
use aluminium::{DrawCallback, Mesh, MeshDesc, PresentPass, RasterPipeline, RasterPipelineDesc, ShaderType, Transform, TransformDesc, WorldRenderer};
use bytemuck::{Pod, Zeroable};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};
use winit::*;

mod gltf_loader;
pub use gltf_loader::{GltfModel, load_gltf};

#[derive(Default)]
struct App {
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

                let _ = world.draw_frame(|graph| {
                    // graph.add_pass(
                    //     PresentPass::new("Simple")
                    //         .execute(unsafe { DrawCallback::new(|ctx| {
                    //             ctx.bind_pipeline(pipe);
                    //             ctx.draw(3);
                    //         })
                    //     }));
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

        let mut world = WorldRenderer::new(&window).expect("Error create world renderer");

        let triangle_mesh = vec![
            Vertex {
                pos: [0.0, 0.5, 0.0],
                color: [0.0, 1.0, 0.0],
            },
            Vertex {
                pos: [-0.5, -0.5, 0.0],
                color: [0.0, 0.0, 1.0],
            },
            Vertex {
                pos: [0.5, -0.5, 0.0],
                color: [0.0, 0.0, 1.0],
            },
        ];

        let transform = world.create::<Transform>(TransformDesc::identity()).unwrap();
        let mesh = world.create::<Mesh>(MeshDesc::new(&triangle_mesh)).expect("Error create simple mesh");

        // let pipeline = world.create::<RasterPipeline>(
        // RasterPipelineDesc::new()
        //         .vertex_shader("../shaders/spv/raster_ps-hlsl.spv")
        //         .fragment_shader("../shaders/spv/raster_ps-hlsl.spv")
        //         .vertex_attribute(ShaderType::Float3)
        //         .vertex_attribute(ShaderType::Float3),
        // ).expect("Error create Raster Pipeline");

        self.world = Some(world);
        self.window = Some(window);
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    unsafe { std::env::set_var("RUST_LOG", "Info"); }

    env_logger::builder().init();

    let event_loop = EventLoop::new()?;
    event_loop.run_app(&mut App::default())?;

    Ok(())
}
