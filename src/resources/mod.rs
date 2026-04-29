use std::marker::PhantomData;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Weak};

use ash::vk;
use parking_lot::RwLock;
use slotmap::{new_key_type, SlotMap};

use crate::bindless::Bindless;
use crate::core::{DescriptorSetLayoutBuilder, Device, ImageBuilder, ImageViewBuilder};
use crate::render_context::RenderContext;
use crate::{VulkanResult};

mod texture;
pub use texture::*;

mod descriptor_manager;
pub use descriptor_manager::*;

mod transform;
pub use transform::{Transform, TransformDesc, TransformPool};

mod pool;
pub use pool::{LinearPool, Pool};

mod mesh;
pub use mesh::*;

mod pipeline_cache;
pub use pipeline_cache::*;

new_key_type! {
    #[allow(missing_docs)]
    pub struct ResourceKey;
}

pub struct Ref<'a, T>(pub(crate) parking_lot::RwLockReadGuard<'a, T>);

pub struct RefMut<'a, T>(pub(crate) parking_lot::RwLockWriteGuard<'a, T>);

impl<'a, T> std::ops::Deref for Ref<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.0
    }
}

impl<'a, T> std::ops::Deref for RefMut<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.0
    }
}

impl<'a, T> std::ops::DerefMut for RefMut<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

/// Unique identifier of the resource with delayed deletion
pub struct Res<T: Destroy> {
    pub(crate) key: ResourceKey,
    pub(crate) ref_count: Arc<AtomicUsize>,
    pub(crate) ctx: Weak<RenderContext>,
    pub(crate) resources: Weak<Resources>,
    pub(crate) _marker: PhantomData<T>,
}

impl<T: Destroy> Clone for Res<T> {
    fn clone(&self) -> Self {
        self.ref_count.fetch_add(1, Ordering::Release);
        Self {
            key: self.key,
            ref_count: self.ref_count.clone(),
            ctx: self.ctx.clone(),
            resources: self.resources.clone(),
            _marker: PhantomData,
        }
    }
}

impl<T: Destroy> Drop for Res<T> {
    fn drop(&mut self) {
        let ref_count = self.ref_count.fetch_sub(1, Ordering::AcqRel);
        if ref_count == 1 {
            T::destroy(self.key, self.ctx.clone(), self.resources.clone());
        }
    }
}

#[allow(missing_docs)]
pub trait Create: Sized + Destroy {
    type Desc<'a>;
    fn create(
        ctx: &Arc<RenderContext>,
        resources: &Arc<Resources>,
        desc: Self::Desc<'_>,
    ) -> VulkanResult<Res<Self>>;
}

#[allow(missing_docs)]
pub trait GetMut: Sized + Destroy {
    fn get_mut<'a>(resources: &'a Resources, res: &Res<Self>) -> RefMut<'a, Self>;
}

#[allow(missing_docs)]
pub trait Get: Sized + Destroy {
    fn get<'a>(resources: &'a Resources, res: &Res<Self>) -> Ref<'a, Self>;
}

#[allow(missing_docs)]
pub trait Destroy: Sized {
    fn destroy(key: ResourceKey, ctx: Weak<RenderContext>, resources: Weak<Resources>);
}

pub struct StorageBuffer;
pub struct StorageTexture;

pub struct Resources {
    pub(crate) bindless: Bindless,
    pub(crate) descriptors: DescriptorManager,
    pub(crate) set: vk::DescriptorSet,
    pub(crate) layout: vk::DescriptorSetLayout,
    pub(crate) indices: RwLock<SlotMap<ResourceKey, IndexBuffer>>,
    pub(crate) vertices: RwLock<SlotMap<ResourceKey, VertexBuffer>>,
    pub(crate) textures: RwLock<SlotMap<ResourceKey, Texture>>,
    pub(crate) transient_textures: RwLock<SlotMap<ResourceKey, TransientTexture>>,
    pub(crate) transforms: RwLock<TransformPool>,
    pub(crate) pipeline_cache: RwLock<PipelineCache>,
}

impl Resources {
    pub fn new(ctx: &Arc<RenderContext>) -> VulkanResult<Arc<Self>> {

        let pipeline_cache = PipelineCache::new();
        let transforms = TransformPool::new(&ctx.device, ctx.frame_in_flight())?;
        let bindless = Bindless::new(&ctx)?;
        let descriptors = DescriptorManager::new(&ctx.device)?;

        let layout = DescriptorSetLayoutBuilder::new(&ctx.device)
            .bindings(vec![
                vk::DescriptorSetLayoutBinding::default()
                    .binding(0)
                    .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                    .descriptor_count(1)
                    .stage_flags(vk::ShaderStageFlags::ALL),
            ])
            .build()?;

        let set = descriptors
            .pool
            .create_descriptor_set(&ctx.device, &[layout.raw])?[0];

        let buffer_info = vk::DescriptorBufferInfo::default()
            .buffer(transforms.buffer.buffers[0].raw)
            .offset(0)
            .range(vk::WHOLE_SIZE);

        let write = vk::WriteDescriptorSet::default()
            .dst_set(set)
            .dst_binding(0)
            .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
            .buffer_info(std::slice::from_ref(&buffer_info));

        unsafe {
            ctx.device.update_descriptor_sets(&[write], &[]);
        }

        Ok(Arc::new(Self {
            bindless,
            descriptors,
            layout: layout.raw,
            set,
            pipeline_cache: RwLock::new(pipeline_cache),
            transforms: RwLock::new(transforms),
            textures: RwLock::new(SlotMap::with_key()),
            transient_textures: RwLock::new(SlotMap::with_key()),
            vertices: RwLock::new(SlotMap::with_key()),
            indices: RwLock::new(SlotMap::with_key()),
        }))
    }

    pub fn create_transient(self: &Arc<Self>, ctx: &Arc<RenderContext>, desc: TransientTextureDesc) -> VulkanResult<Res<TransientTexture>> 
    {
        let extent = ctx.resolution();

        let (image, view) = match desc.format {
            TextureFormat::Depth => { 

                let image = ImageBuilder::new(&ctx.device)
                    .usage(vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT | vk::ImageUsageFlags::SAMPLED)
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
                            .level_count(1)
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
                            .level_count(1)
                    )
                    .view_type(vk::ImageViewType::TYPE_2D)
                    .build()?;

                (image, image_view)
            },
            _ => { todo!() }
        };

        let key = self.transient_textures.try_write().expect("Error lock").insert(TransientTexture {
          image,
          view
        });

        Ok(self.make_handle(ctx, key))
    }

    fn make_handle<T: Destroy>(
        self: &Arc<Self>,
        ctx: &Arc<RenderContext>,
        key: ResourceKey,
    ) -> Res<T> {
        Res {
            key,
            ref_count: Arc::new(AtomicUsize::new(1)),
            ctx: Arc::downgrade(ctx),
            resources: Arc::downgrade(self),
            _marker: PhantomData,
        }
    }

    pub fn update(&self, image_index: u32) {
        self.transforms.write().update(0).unwrap();
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
        self.transforms.write().destroy(device);
    }
}
