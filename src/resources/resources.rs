use std::marker::PhantomData;
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;

use ash::vk;
use parking_lot::RwLock;
use slotmap::SlotMap;

use crate::core::{DescriptorSetLayoutBuilder, Device, ImageBuilder, ImageViewBuilder};
use crate::ext::bindless::Bindless;
use crate::render_context::RenderContext;
use crate::resources::{
    DescriptorManager, IndexBuffer, PipelineCache, Pool, Res, ResourceKey, StorageBuffer,
    TransientTextureDesc, UniformBuffer, VertexBuffer,
};
use crate::{Texture, TextureFormat, TransientTexture, VulkanResult};

pub struct Resources {
    pub(crate) bindless: Bindless,
    pub(crate) descriptors: DescriptorManager,
    pub(crate) set: vk::DescriptorSet,
    pub(crate) layout: vk::DescriptorSetLayout,
    pub(crate) indices: Pool<IndexBuffer>,
    pub(crate) vertices: Pool<VertexBuffer>,
    pub(crate) textures: RwLock<SlotMap<ResourceKey, Texture>>,
    pub(crate) uniforms: RwLock<SlotMap<ResourceKey, UniformBuffer>>,
    pub(crate) storage_buffers: RwLock<SlotMap<ResourceKey, StorageBuffer>>,
    pub(crate) transient_textures: RwLock<SlotMap<ResourceKey, TransientTexture>>,
    pub(crate) pipeline_cache: RwLock<PipelineCache>,
}

impl Resources {
    pub fn new(ctx: &Arc<RenderContext>) -> VulkanResult<Arc<Self>> {
        let pipeline_cache = PipelineCache::new();
        let bindless = Bindless::new(&ctx)?;
        let descriptors = DescriptorManager::new(&ctx.device)?;

        let layout = DescriptorSetLayoutBuilder::new(&ctx.device)
            .bindings(vec![vk::DescriptorSetLayoutBinding::default()
                .binding(0)
                .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::ALL)])
            .build()?;

        let set = descriptors
            .pool
            .create_descriptor_set(&ctx.device, &[layout.raw])?[0];

        // let buffer_info = vk::DescriptorBufferInfo::default()
        //     .buffer(transforms.buffer.buffers[0].raw)
        //     .offset(0)
        //     .range(vk::WHOLE_SIZE);

        // let write = vk::WriteDescriptorSet::default()
        //     .dst_set(set)
        //     .dst_binding(0)
        //     .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
        //     .buffer_info(std::slice::from_ref(&buffer_info));

        // unsafe {
        //     ctx.device.update_descriptor_sets(&[write], &[]);
        // }

        Ok(Arc::new(Self {
            bindless,
            descriptors,
            layout: layout.raw,
            set,
            pipeline_cache: RwLock::new(pipeline_cache),
            uniforms: RwLock::new(SlotMap::with_key()),
            storage_buffers: RwLock::new(SlotMap::with_key()),
            textures: RwLock::new(SlotMap::with_key()),
            transient_textures: RwLock::new(SlotMap::with_key()),
            vertices: Pool::new(),
            indices: Pool::new(),
        }))
    }

    pub fn create_transient(
        self: &Arc<Self>,
        ctx: &Arc<RenderContext>,
        desc: TransientTextureDesc,
    ) -> VulkanResult<Res<TransientTexture>> {
        let extent = ctx.resolution();

        let (image, view) = match desc.format {
            TextureFormat::Depth => {
                let image = ImageBuilder::new(&ctx.device)
                    .usage(
                        vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT
                            | vk::ImageUsageFlags::SAMPLED,
                    )
                    .array_layers(1)
                    .extent(extent.into())
                    .format(vk::Format::D32_SFLOAT)
                    .image_type(vk::ImageType::TYPE_2D)
                    .build()?;

                let image_view = ImageViewBuilder::new(&ctx.device)
                    .image(image.raw)
                    .format(vk::Format::D32_SFLOAT)
                    .subresource_range(
                        vk::ImageSubresourceRange::default()
                            .aspect_mask(vk::ImageAspectFlags::DEPTH)
                            .base_array_layer(0)
                            .layer_count(1)
                            .base_mip_level(0)
                            .level_count(1),
                    )
                    .view_type(vk::ImageViewType::TYPE_2D)
                    .build()?;

                (image, image_view)
            },
            TextureFormat::Color => {
                let image = ImageBuilder::new(&ctx.device)
                    .usage(vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::SAMPLED)
                    .array_layers(1)
                    .extent(extent.into())
                    .format(vk::Format::R8G8B8A8_SRGB)
                    .image_type(vk::ImageType::TYPE_2D)
                    .build()?;

                let image_view = ImageViewBuilder::new(&ctx.device)
                    .image(image.raw)
                    .format(vk::Format::R8G8B8A8_SRGB)
                    .subresource_range(
                        vk::ImageSubresourceRange::default()
                            .aspect_mask(vk::ImageAspectFlags::COLOR)
                            .base_array_layer(0)
                            .layer_count(1)
                            .base_mip_level(0)
                            .level_count(1),
                    )
                    .view_type(vk::ImageViewType::TYPE_2D)
                    .build()?;

                (image, image_view)
            },
            _ => {
                todo!()
            },
        };

        let key = self
            .transient_textures
            .try_write()
            .expect("Error lock")
            .insert(TransientTexture { image, view });

        Ok(self.make_handle(ctx, key))
    }

    pub(crate) fn make_handle<T>(
        self: &Arc<Self>,
        ctx: &Arc<RenderContext>,
        key: ResourceKey,
    ) -> Res<T> {
        Res {
            key,
            ref_count: Arc::new(AtomicUsize::new(1)),
            ctx: Arc::downgrade(ctx),
            resources: self.clone(),
            _marker: PhantomData,
        }
    }

    pub fn update(&self, image_index: u32) {
        // self.transforms.write().update(0).unwrap();
    }

    /// Always Set 0
    pub fn bindless_set(&self) -> vk::DescriptorSet {
        self.bindless.set
    }

    /// Always Set 1
    pub fn per_frame_set(&self) -> vk::DescriptorSet {
        self.set
    }

    pub(crate) fn destroy(&self, device: &Device) {
        self.bindless.destroy(device);
        // self.transforms.write().destroy(device);
    }
}
