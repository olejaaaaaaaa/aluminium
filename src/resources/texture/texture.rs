use crate::{GpuBuffer, Image, ImageView, Sampler, VulkanResult};
use crate::Device;

const MAX_TEXTURE: usize = 100000;

#[derive(Clone, Copy)]
pub struct TextureHandle(usize);

pub struct Texture {
    image: Image,
    view: ImageView,
    sampler: Sampler
}





// pub struct TextureCollection {
//     pub is_dirty: bool,
//     pub data: Vec<Texture>,
//     pub buffer: GpuBuffer,
// }

// impl TextureCollection {

//     pub fn destroy(&mut self, device: &Device) {
//         unsafe {
//             device.allocator.destroy_buffer(self.buffer.raw, &mut self.buffer.allocation);
//         }
//     }

//     pub fn new(device: &Device) -> VulkanResult<Self> {

//         let count = MAX_TEXTURE;
//         let size = (size_of::<Transform>() * count) as u64;

//         let mut buffer = GpuBufferBuilder::cpu_only(&device)
//             .usage(vk::BufferUsageFlags::STORAGE_BUFFER)
//             .size(size)
//             .build()?;

//         let mut data = Vec::with_capacity(count);

//         for _ in 0..count {
//             data.push(Transform::identity());
//         }

//         buffer.upload_data(device, &data)?;

//         Ok(Self { 
//             data,
//             buffer,
//             is_dirty: false
//         })
//     }

//     pub fn get_mut(&mut self, handle: &TransformHandle) -> &mut Transform {
//         self.is_dirty = true;
//         &mut self.data[handle.0]
//     }

//     pub fn get(&self, handle: &TransformHandle) -> &Transform {
//         &self.data[handle.0]
//     }

//     pub fn create_transform(&mut self, data: Transform) -> TransformHandle {
//         self.is_dirty = true;
//         let index = self.data.len();
//         self.data.push(data);
//         TransformHandle(index)
//     }

//     pub fn update(&mut self, device: &Device) -> VulkanResult<()> {
//         if self.is_dirty {
//             self.is_dirty = false;
//             return self.buffer.upload_data(device, &self.data);
//         }
//         Ok(())
//     }
// }
