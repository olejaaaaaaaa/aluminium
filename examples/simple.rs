#![allow(missing_docs)]

use std::error::Error;

use aluminium::types::Vertex;
use aluminium::{
    Material, PresentPass, PresentPassBuilder, Renderable, Transform, WorldRenderer
};

use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};
use winit::*;

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
                world.draw_frame().expect("Error draw frame");
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
            Vertex { pos: [-0.8, -0.8, 0.0], color: [1.0, 0.0, 0.0] },
            Vertex { pos: [ 0.8, -0.8, 0.0], color: [0.0, 1.0, 0.0] },
            Vertex { pos: [ 0.0,  0.8, 0.0], color: [0.0, 0.0, 1.0] },
        ];

        let mesh = world.create_mesh(&triangle_mesh, None).unwrap();
        let material = world.create_material(Material::new()).unwrap();
        let transform = world.create_transform(Transform::identity()).unwrap();
        let _ = world.create_renderable(Renderable::new(mesh, material, transform));

        world.graph_mut().add_pass(
            PresentPassBuilder::new()
                .vertex(r"shaders\spv\raster_vs-hlsl.spv")
                .fragment(r"shaders\spv\raster_ps-hlsl.spv")
                .execute(|ctx, renderables| {
                    ctx.bind_bindless();
                    ctx.bind_pipeline();
                    for i in renderables {
                        ctx.draw_mesh(i);
                    }
                })
                .build()
        );

        self.world = Some(world);
        self.window = Some(window);
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    unsafe {
        std::env::set_var("RUST_LOG", "Info");
    }

    env_logger::builder().init();

    let event_loop = EventLoop::new()?;
    event_loop.run_app(&mut App::default())?;

    Ok(())
}
