use crate::{Handle, PassContext, TransientTexture, core::{FrameBuffer, RenderPass}};
use ash::vk;

pub struct CompiledRasterPass<'frame> {
    name: String,
    uniforms: Vec<vk::DescriptorSet>,
    render_pass: RenderPass,
    imageless_framebuffer: FrameBuffer,
    color_attachments: Vec<Handle<TransientTexture>>,
    depth_attachment: Option<Handle<TransientTexture>>,
    sync: Option<Box<dyn FnOnce(vk::CommandBuffer) + 'frame>>,
    execute: Option<Box<dyn FnOnce(&mut PassContext) + Send + 'frame>>,
}

#[derive(Default)]
pub struct CompiledRasterPassBuilder<'frame> {
    name: Option<String>,
    uniforms: Option<Vec<vk::DescriptorSet>>,
    render_pass: Option<RenderPass>,
    imageless_framebuffer: Option<FrameBuffer>,
    color_attachments: Option<Vec<Handle<TransientTexture>>>,
    depth_attachment: Option<Handle<TransientTexture>>,
    sync: Option<Box<dyn FnOnce(vk::CommandBuffer) + 'frame>>,
    execute: Option<Box<dyn FnOnce(&mut PassContext) + Send + 'frame>>
}

impl<'a> CompiledRasterPassBuilder<'a> {

    pub fn new() -> Self {
        Self::default()
    }

    fn build(self) -> CompiledRasterPass<'a> {

        let name = self.name.expect("Missing Pass name");
        let uniforms = self.uniforms.expect("Missing Descriptor sets");
        let render_pass = self.render_pass.expect("Missing RenderPass");
        let framebuffer = self.imageless_framebuffer.expect("Missing Framebuffer");
        let color_attachments = self.color_attachments.expect("Missing Color Attachments");
        let sync = self.sync.expect("Missing begin sync closure");
        let execute = self.execute.expect("Missing execute closure");
        let depth_attachment = self.depth_attachment;

        assert_ne!(color_attachments.is_empty(), depth_attachment.is_none(), "Raster Pass: {} required minimum one color attachment or depth attachment", name);

        CompiledRasterPass {
            name, 
            uniforms, 
            render_pass, 
            imageless_framebuffer: framebuffer, 
            color_attachments, 
            depth_attachment, 
            sync: Some(sync),
            execute: Some(execute), 
        }
    }
}