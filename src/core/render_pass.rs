use ash::vk;
use log::debug;
use puffin::profile_scope;

use super::device::Device;
use super::{Subpass, SubpassDesc, VulkanError, VulkanResult};

pub struct RenderPass {
    pub(crate) raw: vk::RenderPass,
}

impl RenderPass {
    pub fn destroy(&self, device: &Device) {
        unsafe {
            device.destroy_render_pass(self.raw, None);
        }
    }
}

pub struct RenderPassBuilder<'a> {
    pub device: &'a Device,
    pub attachments: Option<Vec<vk::AttachmentDescription>>,
    pub dependencies: Option<Vec<vk::SubpassDependency>>,
    pub subpasses: Option<Vec<Subpass>>,
}

impl<'a> RenderPassBuilder<'a> {
    pub fn default(device: &'a Device, color: vk::Format, depth: vk::Format) -> Self {
        let subpass = Subpass::new(
            SubpassDesc::empty()
                .add_color_attachment_ref(
                    vk::AttachmentReference::default()
                        .attachment(0)
                        .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL),
                )
                .add_depth_attachment_ref(
                    vk::AttachmentReference::default()
                        .attachment(1)
                        .layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL),
                )
                .with_bind_point(vk::PipelineBindPoint::GRAPHICS),
        );

        RenderPassBuilder {
            device,
            attachments: Some(vec![
                vk::AttachmentDescription {
                    flags: vk::AttachmentDescriptionFlags::empty(),
                    format: color,
                    samples: vk::SampleCountFlags::TYPE_1,
                    load_op: vk::AttachmentLoadOp::CLEAR,
                    store_op: vk::AttachmentStoreOp::STORE,
                    stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
                    stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
                    initial_layout: vk::ImageLayout::UNDEFINED,
                    final_layout: vk::ImageLayout::PRESENT_SRC_KHR,
                },
                vk::AttachmentDescription::default()
                    .format(depth)
                    .samples(vk::SampleCountFlags::TYPE_1)
                    .load_op(vk::AttachmentLoadOp::CLEAR)
                    .store_op(vk::AttachmentStoreOp::DONT_CARE)
                    .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
                    .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
                    .initial_layout(vk::ImageLayout::UNDEFINED)
                    .final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL),
            ]),
            dependencies: Some(vec![vk::SubpassDependency {
                src_subpass: vk::SUBPASS_EXTERNAL,
                dst_subpass: 0,
                src_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
                    | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
                dst_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
                    | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
                src_access_mask: vk::AccessFlags::empty(),
                dst_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_WRITE
                    | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
                dependency_flags: vk::DependencyFlags::BY_REGION,
            }]),
            subpasses: Some(vec![subpass]),
        }
    }

    pub fn build(self) -> VulkanResult<RenderPass> {
        profile_scope!("Render Pass");

        let device = self.device;
        let raw_subpasses = self
            .subpasses
            .unwrap()
            .iter()
            .map(|x| x.raw)
            .collect::<Vec<_>>();

        let binding1 = self.attachments.unwrap_or(vec![]);
        let binding2 = self.dependencies.unwrap_or(vec![]);

        let create_info = vk::RenderPassCreateInfo::default()
            .attachments(&binding1)
            .dependencies(&binding2)
            .subpasses(&raw_subpasses);

        debug!("Render Pass: {:?}", create_info);

        let render_pass = unsafe {
            device
                .create_render_pass(&create_info, None)
                .map_err(VulkanError::Unknown)?
        };

        Ok(RenderPass { raw: render_pass })
    }
}
