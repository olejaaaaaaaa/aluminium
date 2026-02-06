
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

new_key_type! { pub struct PipelineLayoutHandle; }
new_key_type! { pub struct SamplerHandle; }
new_key_type! { pub struct RasterPipelineHandle; }

new_key_type! { pub struct FrameBufferHandle; }

pub struct ResourceManager {
    mesh: MeshCollection,
    material: MaterialCollection,
    transform: TransformCollection,
    renderable: RenderableCollection,
    raster_pipeline: SlotMap<RasterPipelineHandle, GraphicsPipeline>,
    pipeline_layout: SlotMap<PipelineLayoutHandle, PipelineLayout>,
    frame_buffer: SlotMap<FrameBufferHandle, FrameBuffer>,
    #[allow(dead_code)]
    sampler: SlotMap<SamplerHandle, Sampler>,
}

impl ResourceManager {
    pub fn destroy(&self, _device: &Device) {}

    pub fn new(ctx: &RenderContext) -> VulkanResult<Self> {
        Ok(ResourceManager {
            mesh: MeshCollection::new(),
            material: MaterialCollection::new(),
            transform: TransformCollection::new(&ctx.device)?,
            renderable: RenderableCollection::new(),
            sampler: SlotMap::with_key(),
            frame_buffer: SlotMap::with_key(),
            pipeline_layout: SlotMap::with_key(),
            raster_pipeline: SlotMap::with_key(),
        })
    }

    pub fn add_raster_pipeline(&mut self, pipeline: GraphicsPipeline) -> RasterPipelineHandle {
        self.raster_pipeline.insert(pipeline)
    }

    pub fn add_layout(&mut self, pipeline: PipelineLayout) -> PipelineLayoutHandle {
        self.pipeline_layout.insert(pipeline)
    }

    pub fn get_layout(&mut self, pipeline: PipelineLayoutHandle) -> Option<&PipelineLayout> {
        self.pipeline_layout.get(pipeline)
    }

    pub fn get_raster_pipeline(&self, pipeline: RasterPipelineHandle) -> Option<&GraphicsPipeline> {
        self.raster_pipeline.get(pipeline)
    }

    pub fn get_framebuffer(&self, frame_buffer: FrameBufferHandle) -> Option<&FrameBuffer> {
        self.frame_buffer.get(frame_buffer)
    }

    pub fn create_static_mesh_immediately<
        T: AttributeDescriptions + BindingDescriptions + Pod + Zeroable,
    >(
        &mut self,
        ctx: &RenderContext,
        mesh: &[T],
        indices: Option<&[u32]>,
    ) -> VulkanResult<MeshHandle> {
        let size = (size_of::<T>() + mesh.len()) as u64;

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

        Ok(self.mesh.add_mesh(Mesh {
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
        self.renderable.add_renderable(renderable)
    }

    pub fn create_material(&mut self, material: Material) -> MaterialHandle {
        self.material.add_material(material)
    }

    pub fn create_transform(&mut self, transform: Transform) -> TransformHandle {
        self.transform.create_transform(transform)
    }
}
