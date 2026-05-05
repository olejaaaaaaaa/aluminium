use std::collections::HashMap;

use aluminium::types::Vertex;
use aluminium::{
    Layout, RasterPipeline, RasterPipelineDesc, ShaderType, VertexInput, VulkanResult,
    WorldRenderer,
};
use bytemuck::{Pod, Zeroable};
use egui::epaint::{ImageDelta, Primitive};
use egui::{ClippedPrimitive, ImageData, TextureId, ViewportId};
use winit::event::WindowEvent;
use winit::window::Window;

/// Vulkan renderer for egui.
pub struct UiRenderer {
    egui_ctx: egui::Context,
    egui_state: egui_winit::State,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct UiVertex {
    pos: [f32; 2],
    uv: [f32; 2],
    color: [f32; 4],
}

impl Layout for UiVertex {
    fn layout() -> VertexInput {
        VertexInput::new()
            .attr("pos", ShaderType::Float2)
            .attr("uv", ShaderType::Float2)
            .attr("color", ShaderType::Float4)
    }
}

impl UiRenderer {
    pub fn new(world: &WorldRenderer, window: &Window) -> VulkanResult<Self> {
        let egui_ctx = egui::Context::default();
        let egui_state = egui_winit::State::new(
            egui_ctx.clone(),
            ViewportId::ROOT,
            &window,
            None,
            None,
            None,
        );

        // let pipeline = world.create::<RasterPipeline>(
        // RasterPipelineDesc::new()
        //         .vertex_shader("./shaders/spv/egui_vert.spv")
        //         .fragment_shader("./shaders/spv/egui_frag.spv")
        //         .depth_test(false)
        //         .dynamic_scissors(true)
        //         .dynamic_viewport(true)
        // )?;

        Ok(Self {
            egui_ctx,
            egui_state,
        })
    }

    pub fn on_window_event(&self, window: &Window, event: &WindowEvent) {}

    pub fn draw(&mut self, window: &Window) {
        let raw_input = self.egui_state.take_egui_input(&window);

        let egui::FullOutput {
            platform_output,
            textures_delta,
            shapes,
            pixels_per_point,
            ..
        } = self.egui_ctx.run_ui(raw_input, |ctx| {});

        self.egui_state
            .handle_platform_output(&window, platform_output);
        let clipped_primitives = self.egui_ctx.tessellate(shapes, pixels_per_point);
    }
}

/// Orthographic projection matrix for use with Vulkan.
///
/// This matrix is meant to be used when the source coordinate space is
/// right-handed and y-up (the standard computer graphics coordinate space)and
/// the destination space is right-handed and y-down, with Z (depth) clip
/// extending from 0.0 (close) to 1.0 (far).
///
/// from: https://github.com/fu5ha/ultraviolet (to limit dependencies)
#[inline]
pub fn orthographic_vk(
    left: f32,
    right: f32,
    bottom: f32,
    top: f32,
    near: f32,
    far: f32,
) -> [f32; 16] {
    let rml = right - left;
    let rpl = right + left;
    let tmb = top - bottom;
    let tpb = top + bottom;
    let fmn = far - near;

    #[rustfmt::skip]
    let res = [
        2.0 / rml, 0.0, 0.0, 0.0,
        0.0, -2.0 / tmb, 0.0, 0.0,
        0.0, 0.0, -1.0 / fmn, 0.0,
        -(rpl / rml), -(tpb / tmb), -(near / fmn), 1.0
    ];

    res
}
