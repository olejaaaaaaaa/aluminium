#![allow(missing_docs)]

use std::error::Error;

use aluminium::{
    Mesh, MeshDesc, PresentPass, RasterPipeline, RasterPipelineDesc, Res, Scissor, ShaderType, Transform, TransformDesc, VertexInput, Viewport,
    WorldRenderer,
};
use tracing_subscriber::filter::LevelFilter;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};
use winit::*;

mod ui;
pub use ui::Ui;

mod gltf_loader;
pub use gltf_loader::{GltfModel, load_gltf};

#[derive(Default)]
struct App {
    model: Option<GltfModel>,
    pipeline: Option<Res<RasterPipeline>>,
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
                let pipeline = self.pipeline.as_ref().unwrap().clone();
                let model = self.model.as_ref().unwrap().clone();

                let _ = world.draw_frame(move |graph| {
                    graph.add_pass(PresentPass::new("Final Pass").execute(move |ctx| unsafe {
                        ctx.set_viewport(Viewport::FullRes);
                        ctx.set_scissor(Scissor::FullRes);
                        ctx.bind_pipeline(pipeline);
                        for i in &model.meshes {
                            ctx.draw_mesh(i.clone());
                        }
                    }));
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

        let pipeline = world
            .create::<RasterPipeline>(
                RasterPipelineDesc::new()
                    .vertex_shader(r"C:\Users\Oleja\Desktop\aluminium\shaders\spv\raster_vs.spv")
                    .fragment_shader(r"C:\Users\Oleja\Desktop\aluminium\shaders\spv\raster_ps.spv")
                    .vertex_input(
                        VertexInput::new()
                            .with(ShaderType::Float3)
                            .with(ShaderType::Float3),
                    )
                    .dynamic_scissors(true)
                    .dynamic_viewport(true),
            )
            .expect("Error create pipeline");

        let model = load_gltf(
            &mut world,
            r"C:\Users\Oleja\Desktop\aluminium\examples\simple\assets\flighthelmet\scene.gltf",
        )
        .expect("Error load gltf model");

        self.model = Some(model);
        self.pipeline = Some(pipeline);
        self.world = Some(world);
        self.window = Some(window);
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .with_target(false)
        .with_max_level(LevelFilter::DEBUG)
        .init();

    let event_loop = EventLoop::new()?;
    event_loop.run_app(&mut App::default())?;

    Ok(())
}
