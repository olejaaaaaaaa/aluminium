
use ash::vk;
use bytemuck::{Pod, Zeroable};
use slotmap::*;

use crate::core::{
    AttributeDescriptions, BindingDescriptions, Device, FrameBuffer, GpuBufferBuilder,
    GraphicsPipeline, PipelineLayout, Sampler, VulkanResult,
};
use crate::render_context::RenderContext;

// mod aabb;
// pub use aabb::*;

mod mesh;
pub use mesh::*;

mod materials;
pub use materials::*;

mod transforms;
pub use transforms::*;

mod renderable;
pub use renderable::*;

new_key_type! { 
    pub struct PipelineLayoutHandle; 
    pub struct SamplerHandle;
    pub struct RasterPipelineHandle;
    pub struct FrameBufferHandle;
}

pub struct AssetManager {
    pub(crate) mesh: MeshCollection,
    pub(crate) material: MaterialCollection,
    pub(crate) transform: TransformCollection,
    pub(crate) renderable: RenderableCollection,
}

impl AssetManager {
    pub fn new(device: &Device) -> VulkanResult<Self> {
        Ok(Self {
            mesh: MeshCollection::new(),
            material: MaterialCollection::new(),
            transform: TransformCollection::new(device)?,
            renderable: RenderableCollection::new()
        })
    }
}

pub struct LowLevelManager {
    raster_pipeline: SlotMap<RasterPipelineHandle, GraphicsPipeline>,
    pipeline_layout: SlotMap<PipelineLayoutHandle, PipelineLayout>,
    frame_buffer: SlotMap<FrameBufferHandle, FrameBuffer>,
    sampler: SlotMap<SamplerHandle, Sampler>,
}

impl LowLevelManager {
    pub fn new() -> Self {
        Self { 
            raster_pipeline: SlotMap::with_key(), 
            pipeline_layout: SlotMap::with_key(), 
            frame_buffer: SlotMap::with_key(), 
            sampler: SlotMap::with_key() 
        }
    }
}

pub struct ResourceManager {
    pub(crate) assets: AssetManager,
    pub(crate) low_level: LowLevelManager,
}

impl ResourceManager {
    pub fn destroy(&self, _device: &Device) {}

    pub fn new(device: &Device) -> VulkanResult<Self> {
        Ok(Self {
            assets: AssetManager::new(device)?,
            low_level: LowLevelManager::new()
        })
    }

    pub fn get_renderables(&self) -> Vec<Renderable> {
        self.assets.renderable.get_renderables().clone()
    }

    pub fn get_mesh(&self, handle: MeshHandle) -> &Mesh {
        self.assets.mesh.get_mesh(handle)
    }

    pub fn add_raster_pipeline(&mut self, pipeline: GraphicsPipeline) -> RasterPipelineHandle {
        self.low_level.raster_pipeline.insert(pipeline)
    }

    pub fn add_layout(&mut self, pipeline: PipelineLayout) -> PipelineLayoutHandle {
        self.low_level.pipeline_layout.insert(pipeline)
    }

    pub fn get_layout(&mut self, pipeline: PipelineLayoutHandle) -> Option<&PipelineLayout> {
        self.low_level.pipeline_layout.get(pipeline)
    }

    pub fn get_raster_pipeline(&self, pipeline: RasterPipelineHandle) -> Option<&GraphicsPipeline> {
        self.low_level.raster_pipeline.get(pipeline)
    }

    pub fn get_framebuffer(&self, frame_buffer: FrameBufferHandle) -> Option<&FrameBuffer> {
        self.low_level.frame_buffer.get(frame_buffer)
    }

    pub fn create_static_mesh_immediately<
        T: AttributeDescriptions + BindingDescriptions + Pod + Zeroable,
    >(
        &mut self,
        ctx: &RenderContext,
        mesh: &[T],
        indices: Option<&[u32]>,
    ) -> VulkanResult<MeshHandle> {
        let size = (size_of::<T>() * mesh.len()) as u64;

        let mut vertex_buffer = GpuBufferBuilder::cpu_only(&ctx.device)
            .size(size)
            .usage(vk::BufferUsageFlags::VERTEX_BUFFER)
            .build()?;

        vertex_buffer.upload_data(&ctx.device, &mesh)?;

        let index_buffer = if let Some(indices) = indices {
            let mut index_buffer = GpuBufferBuilder::cpu_only(&ctx.device)
                .size((size_of::<u32>() * indices.len()) as u64)
                .usage(vk::BufferUsageFlags::INDEX_BUFFER)
                .build()?;

            index_buffer.upload_data(&ctx.device, indices)?;

            Some(index_buffer)
        } else {
            None
        };

        Ok(self.assets.mesh.add_mesh(Mesh {
            instance_offset: 0,
            instance_count: 1,
            vertex_offset: 0,
            instance_buffer: None,
            vertex_buffer,
            indices: indices.map(|x| x.to_vec()),
            index_buffer,
        }))
    }

    pub fn create_renderable(&mut self, renderable: Renderable) -> RenderableHandle {
        self.assets.renderable.add_renderable(renderable)
    }

    pub fn create_material(&mut self, material: Material) -> MaterialHandle {
        self.assets.material.add_material(material)
    }

    pub fn create_transform(&mut self, transform: Transform) -> TransformHandle {
        self.assets.transform.create_transform(transform)
    }
}
