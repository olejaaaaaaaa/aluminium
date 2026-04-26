use std::{marker::PhantomData, ops::Add as _};
use crate::{Handle, LoadOp, RenderTarget, Resolution, StoreOp, TextureFormat, TransientTexture, frame_graph::types::{ColorAttachment, DepthAttachment, Location}, frame_scope::{FrameResources, Id, TransientTextureDesc}};

pub struct PassBuilder<'a> {
    pub(crate) render_target: RenderTarget,
    pub(crate) write_textures: Vec<Handle<TransientTexture>>,
    pub(crate) read_textures: Vec<(Handle<TransientTexture>, Location)>,
    pub(crate) resources: &'a mut FrameResources,
}

impl<'a> PassBuilder<'a> {

    pub fn backbuffer(&mut self) -> Handle<TransientTexture> {
        
        let id = self.resources.backbuffers.insert(TransientTextureDesc { 
            name: "BackBuffer", 
            format: TextureFormat::Color, 
            resolution: Resolution::FullRes 
        });

        Handle { 
            id, 
            version: 0, 
            _marker: PhantomData 
        }
    }

    pub fn write_depth(&mut self, texture: Handle<TransientTexture>, load: LoadOp, store: StoreOp) -> Handle<TransientTexture> {

        self.render_target.depth = Some(DepthAttachment {
            depth: texture,
            load,
            store,
        });

        self.write_textures.push(texture);

        Handle { 
            id: texture.id, 
            version: texture.version.add(1), 
            _marker: PhantomData
        }
    }

    pub fn read_texture(&mut self, texture: Handle<TransientTexture>, location: Location) -> Handle<TransientTexture> {
        todo!()
    }
 
    pub fn write_color(&mut self, texture: Handle<TransientTexture>, load: LoadOp, store: StoreOp) -> Handle<TransientTexture> {

        self.render_target.colors.push(ColorAttachment {
            color: texture,
            load,
            store
        });

        self.write_textures.push(texture);

        Handle { 
            id: texture.id, 
            version: texture.version.add(1), 
            _marker: PhantomData
        }
    }

    pub fn create_storage_texture(&mut self, name: &'static str, format: TextureFormat, resolution: Resolution) -> Handle<TransientTexture> {
        todo!()
    }

    pub fn create_storage_buffer(&mut self, name: &'static str, size: u64) -> Handle<TransientTexture> {
        todo!()
    }

    pub fn get_or_create_temporal_texture(&mut self, name: &'static str, format: TextureFormat, resolution: Resolution) -> Handle<TransientTexture> {
        todo!()
    }

    pub fn get_or_create_temporal_buffer(&mut self, name: &'static str, size: u64) -> Handle<TransientTexture> {
        todo!()
    }

    pub fn create_buffer(&mut self, name: &'static str, size: u64) -> Handle<TransientTexture> {
        todo!()
    }

    pub fn create_texture(
        &mut self,
        name: &'static str,
        format: TextureFormat,
        resolution: Resolution,
    ) -> Handle<TransientTexture> {

        let id = self.resources.textures.insert(TransientTextureDesc {
            name,
            format,
            resolution,
        });

        Handle {
            id,
            version: 0,
            _marker: PhantomData,
        }
    }
}