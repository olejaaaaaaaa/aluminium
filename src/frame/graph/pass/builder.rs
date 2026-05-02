use std::{marker::PhantomData, ops::Add as _};
use crate::{Handle, LoadOp, Location, RenderTarget, Resolution, StoreOp, TextureFormat, frame::{Id, scope::FrameResources, types::{ColorAttachment, DepthAttachment}}, resources::{Res, StorageBuffer, TransientTexture, TransientTextureDesc}};

pub struct PassBuilder<'a> {
    pub(crate) render_target: RenderTarget,
    pub(crate) read_storage_buffers: Vec<(*const Res<StorageBuffer>, Location)>,
    pub(crate) write_textures: Vec<Handle<TransientTexture>>,
    pub(crate) read_textures: Vec<(Handle<TransientTexture>, Location)>,
    pub(crate) resources: &'a mut FrameResources,
}

impl<'a> PassBuilder<'a> {

    pub fn backbuffer(&mut self) -> Handle<TransientTexture> {
        // the only handle that will be invalid
        Handle { 
            id: Id::default(), 
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

    pub fn read_storage_buffer(&mut self, buffer: &Res<StorageBuffer>, location: Location) {
        self.read_storage_buffers.push((buffer as *const _, location));
    }

    pub fn read_texture(&mut self, texture: Handle<TransientTexture>, location: Location) {
        self.read_textures.push((texture, location));
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