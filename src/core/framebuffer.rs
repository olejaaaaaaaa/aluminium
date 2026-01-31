use ash::vk::{
    Result, {self},
};
use log::debug;
use puffin::profile_scope;

use super::device::Device;
use super::{VulkanError, VulkanResult};

pub struct FrameBuffer {
    pub(crate) raw: vk::Framebuffer,
}

impl FrameBuffer {
    pub fn destroy(&self, device: &Device) {
        unsafe { device.destroy_framebuffer(self.raw, None) };
    }
}

pub struct FrameBufferBuilder<'a> {
    device: &'a Device,
    attachments: Vec<vk::ImageView>,
    create_info: vk::FramebufferCreateInfo<'static>,
}

impl<'a> FrameBufferBuilder<'a> {
    pub fn new(device: &'a Device, render_pass: vk::RenderPass) -> Self {
        FrameBufferBuilder {
            device,
            attachments: vec![],
            create_info: vk::FramebufferCreateInfo::default().render_pass(render_pass),
        }
    }

    pub fn layers(mut self, layers: u32) -> Self {
        self.create_info = self.create_info.layers(layers);
        self
    }

    pub fn add_attachment(mut self, view: vk::ImageView) -> Self {
        self.attachments.push(view);
        self
    }

    pub fn extent(mut self, extent: vk::Extent2D) -> Self {
        self.create_info = self.create_info.width(extent.width).height(extent.height);
        self
    }

    pub fn render_pass(mut self, pass: vk::RenderPass) -> Self {
        self.create_info = self.create_info.render_pass(pass);
        self
    }

    pub fn build(mut self) -> VulkanResult<FrameBuffer> {
        profile_scope!("FrameBuffer");

        self.create_info.attachment_count = self.attachments.len() as u32;
        self.create_info.p_attachments = self.attachments.as_ptr();

        #[cfg(feature = "runtime-check")]
        {
            let limits = self.device.phys_dev.prop.limits;

            if self.create_info.width > limits.max_framebuffer_width {
                return Err(VulkanError::Unknown(Result::from_raw(0)));
            }

            if self.create_info.height > limits.max_framebuffer_height {
                return Err(VulkanError::Unknown(Result::from_raw(0)));
            }
        }

        debug!("FrameBuffer: {:?}", self.create_info);

        let frame_buffer = unsafe {
            self.device
                .create_framebuffer(&self.create_info, None)
                .map_err(|e| VulkanError::Unknown(e))?
        };

        Ok(FrameBuffer { raw: frame_buffer })
    }
}
