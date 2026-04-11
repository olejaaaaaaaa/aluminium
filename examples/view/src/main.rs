#![allow(missing_docs)]

use std::error::Error;
use std::time::Instant;

use aluminium::{
    BackBuffer, FrameGraphTexture, Handle, PresentPass, RasterPass, RasterPipeline, RasterPipelineDesc, RenderTargetsDesc, Res, Scissor, ShaderType, VertexInput, Viewport, WorldRenderer
};

use tracing_subscriber::filter::LevelFilter;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};
use winit::*;

mod ui;
pub use ui::UiRenderer;

mod gltf_loader;
pub use gltf_loader::{GltfModel, load_gltf};



                    /*
                    
                    Compute Pass
                        Читает:
                            Uniform Buffer
                            Storage Buffer (read-only)
                            Storage Image (read-only)
                            Sampled Image / Texture

                        Пишет:
                            Storage Buffer
                            Storage Image

                    Raster Pass
                        Читает:
                            Uniform Buffer
                            Storage Buffer (read-only)
                            Sampled Image / Texture
                            Input Attachment — да, но только то, что было Output в этом же RenderPass (тот же VkRenderPass, subpass dependency). Это subpass input, не между пассами.

                        Пишет:
                            Color Attachment (RenderTarget) — обязательно хотя бы один, либо depth
                            Depth/Stencil Attachment
                            Storage Buffer / Storage Image

                    Ray Tracing Pass
                        Читает:
                            Acceleration Structure (TLAS) — это его уникальный ресурс
                            Uniform Buffer
                            Storage Buffer
                            Sampled Image / Texture

                        Пишет:
                            Storage Image — главный output, обычно пишет финальную картинку именно так
                            Storage Buffer
                    */


#[derive(Default)]
struct App {
    global_time: Option<std::time::Instant>,
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
                let pipeline = self.pipeline.as_ref().unwrap();
                let model = self.model.as_ref().unwrap();
                let time_sec = self.global_time.as_ref().unwrap().elapsed().as_secs_f32();

                let _ = world.draw_frame(move |graph| {

                    #[derive(Clone, Copy, Default)]
                    pub struct PassData {

                    }

                    graph.add_pass(
                        PresentPass::new(
                            "Final Pass", 
                            |builder| {
                               PassData {}
                            }, 
                            move |ctx, data| unsafe {
                                ctx.bind_pipeline(pipeline);
                                ctx.push_constants([time_sec, 2.0]);
                                ctx.set_viewport(Viewport::FullRes);
                                ctx.set_scissor(Scissor::FullRes);
                                for mesh in &model.meshes {
                                    ctx.draw_mesh(mesh);
                                }
                            }
                        )
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

        let pipeline = world
            .create::<RasterPipeline>(
            RasterPipelineDesc::new()
                    .vertex_shader("./shaders/spv/raster_vs.spv")
                    .fragment_shader("./shaders/spv/raster_ps.spv")
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
            &world,
            "./examples/view/assets/flighthelmet/scene.gltf",
        )
        .expect("Error load gltf model");

        self.global_time = Some(Instant::now());
        self.model = Some(model);
        self.pipeline = Some(pipeline);
        self.world = Some(world);
        self.window = Some(window);
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .with_target(false)
        .with_max_level(LevelFilter::INFO)
        .init();

    let event_loop = EventLoop::new()?;
    event_loop.run_app(&mut App::default())?;

    Ok(())
}
