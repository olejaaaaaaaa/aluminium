use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

use ash::vk;
use puffin::profile_scope;

use super::Device;

pub struct PipelineCache {
    pub raw: vk::PipelineCache,
}

impl PipelineCache {
    pub fn destroy(&self, device: &Device) {
        unsafe {
            device.destroy_pipeline_cache(self.raw, None);
        }
    }

    pub fn new(device: &Device) -> Result<Self, vk::Result> {
        profile_scope!("Pipeline Cache");

        let cache_info = vk::PipelineCacheCreateInfo::default()
            .flags(vk::PipelineCacheCreateFlags::empty())
            .initial_data(&[]);

        let cache = unsafe { device.create_pipeline_cache(&cache_info, None)? };

        Ok(Self { raw: cache })
    }

    pub fn from_file(device: &Device, path: &Path) -> Result<Self, vk::Result> {
        profile_scope!("Load Pipeline Cache from file");

        let initial_data = if path.exists() {
            let mut file = File::open(path).unwrap();
            let mut data = Vec::new();
            file.read_to_end(&mut data).unwrap();
            data
        } else {
            return Err(vk::Result::from_raw(0));
        };

        let cache_info = vk::PipelineCacheCreateInfo::default()
            .flags(vk::PipelineCacheCreateFlags::empty())
            .initial_data(&initial_data);

        let cache = unsafe { device.create_pipeline_cache(&cache_info, None)? };

        Ok(Self { raw: cache })
    }

    pub fn save_to_file(&self, device: &Device, path: &Path) -> Result<(), vk::Result> {
        profile_scope!("Save Pipeline Cache to file");

        let data = unsafe { device.get_pipeline_cache_data(self.raw)? };

        let mut file = File::create(path).unwrap();
        file.write_all(&data).unwrap();

        Ok(())
    }
}
