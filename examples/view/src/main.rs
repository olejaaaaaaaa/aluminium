#![allow(missing_docs)]

use std::error::Error;
use std::time::Instant;

use aluminium::types::PbrVertex;
use aluminium::{Handle, LoadOp, Location, RasterPass, RasterPipeline, RasterPipelineDesc, RenderTarget, Res, Resolution, Scissor, ShaderType, StoreOp, Texture, TextureDesc, TextureFormat, TransientTexture, VertexInput, Viewport, WorldRenderer};
use tracing_subscriber::filter::LevelFilter;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};
use winit::*;

mod view;
pub use view::View;

mod ui;
pub use ui::UiRenderer;

mod gltf_loader;
pub use gltf_loader::{GltfModel, load_gltf};

#[derive(Default)]
struct App {
    view: Option<View>,
    window: Option<winit::window::Window>,
}

impl ApplicationHandler for App {
    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            },
            _ => {
                let view = self.view.as_mut().unwrap();
                view.handle_event(event_loop, id, event);
            },
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

        let view = View::new(&window);
        self.view = Some(view);
        self.window = Some(window);
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .with_target(false)
        .with_max_level(LevelFilter::ERROR)
        .init();

    let event_loop = EventLoop::new()?;
    event_loop.run_app(&mut App::default())?;

    Ok(())
}

