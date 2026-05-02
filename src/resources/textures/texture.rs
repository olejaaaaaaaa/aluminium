use std::sync::Arc;

use ash::vk;
use vk_sync::ImageBarrier;

use crate::{Resolution, VulkanResult};
use crate::core::{CommandPoolBuilder, GpuBufferBuilder, Image, ImageBuilder, ImageView, ImageViewBuilder};
use crate::render_context::RenderContext;
use crate::resources::{Create, Res, ResourceKey, Resources};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum TextureFormat {
    Depth,
    DepthStencil,
    HDR,
    Color,
    Data
}

pub struct Texture {
    pub(crate) index: u32,
    view: ImageView,
    image: Image,
}

impl Create for Texture {
    type Desc<'a> = TextureDesc<'a>;
    fn create(
            ctx: &Arc<RenderContext>,
            resources: &Arc<Resources>,
            desc: Self::Desc<'_>,
        ) -> VulkanResult<Res<Self>> {
        
        let image = ImageBuilder::new(&ctx.device)
            .extent(vk::Extent3D { width: desc.width, height: desc.height, depth: 1 })
            .array_layers(1)
            .format(vk::Format::R8G8B8A8_SRGB)
            .image_type(vk::ImageType::TYPE_2D)
            .usage(vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::TRANSFER_DST)
            .build()?;

        let buffer_size = size_of_val(desc.pixels) as u64;

        let mut staging_buffer = GpuBufferBuilder::cpu_only(&ctx.device)
            .size(buffer_size)
            .usage(vk::BufferUsageFlags::TRANSFER_SRC)
            .build()?;

        staging_buffer.upload_data(desc.pixels)?;

        let pool = CommandPoolBuilder::reset(&ctx.device).build()?;
        let cmd = pool.allocate_cmd_buffers(&ctx.device, vk::CommandBufferLevel::PRIMARY, 1)?[0];

        let barrier = vk::ImageMemoryBarrier::default()
            .old_layout(vk::ImageLayout::UNDEFINED)
            .new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
            .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .src_access_mask(vk::AccessFlags::empty())   
            .dst_access_mask(vk::AccessFlags::TRANSFER_WRITE)
            .image(image.raw)
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            });

        let begin_info = vk::CommandBufferBeginInfo::default()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

        unsafe {
            ctx.device.begin_command_buffer(cmd, &begin_info);
        }

        unsafe {
            ctx.device.cmd_pipeline_barrier(
                cmd,
                vk::PipelineStageFlags::TOP_OF_PIPE,
                vk::PipelineStageFlags::TRANSFER,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[barrier],
            );
        }

        let region = vk::BufferImageCopy::default()
            .buffer_offset(0)
            .buffer_row_length(0)
            .buffer_image_height(0)
            .image_subresource(vk::ImageSubresourceLayers {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                mip_level: 0,
                base_array_layer: 0,
                layer_count: 1,
            })
            .image_offset(vk::Offset3D { x: 0, y: 0, z: 0 })
            .image_extent(vk::Extent3D {
                width: desc.width,
                height: desc.height,
                depth: 1,
            });

        unsafe {
            ctx.device.cmd_copy_buffer_to_image(
                cmd,
                staging_buffer.raw,       
                image.raw,                 
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &[region],
            );
        }

        let barrier2 = vk::ImageMemoryBarrier::default()
            .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
            .new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .image(image.raw)
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            });

        unsafe {
            ctx.device.cmd_pipeline_barrier(
                cmd,
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::FRAGMENT_SHADER,
                vk::DependencyFlags::empty(),
                &[], &[],
                &[barrier2],
            );
        }

        unsafe {
            ctx.device.end_command_buffer(cmd);
        }

        let transfer_queue = ctx.device.queue_pool.graphics().unwrap();

        let binding = [cmd];
        let submit_info = vk::SubmitInfo::default().command_buffers(&binding);
        unsafe {
            let _ = ctx.device.queue_submit(transfer_queue.raw, &[submit_info], vk::Fence::null());
            let _ = ctx.device.queue_wait_idle(transfer_queue.raw);
        }

        let image_view = ImageViewBuilder::new(&ctx.device)
            .components(vk::ComponentMapping::default())
            .format(vk::Format::R8G8B8A8_SRGB)
            .image(image.raw)
            .view_type(vk::ImageViewType::TYPE_2D)
            .subresource_range(
                vk::ImageSubresourceRange::default()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .base_array_layer(0)
                    .layer_count(1)
                    .base_mip_level(0)
                    .level_count(1)
            )   
            .build()?;

        let index = resources.bindless.alloc_texture(&ctx.device, image_view.raw);
        let key = resources.textures.write().insert(Texture {
            index,
            image,
            view: image_view
        });

        Ok(resources.make_handle(ctx, key))
    }
}

pub struct TextureDesc<'a> {
    pub width: u32,
    pub height: u32,
    pub format: TextureFormat,
    pub pixels: &'a [u8]
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct TransientTextureDesc {
    pub name: &'static str,
    pub format: TextureFormat,
    pub resolution: Resolution,
}

pub struct TransientTexture {
    pub(crate) image: Image,
    pub(crate) view: ImageView
}


impl PartialEq for TransientTexture {
    fn eq(&self, other: &Self) -> bool {
        self.image.raw == other.image.raw && self.view.raw == other.view.raw
    }
}