use std::error::Error;
use std::io::Read;
use std::path::Path;

use ash::vk;
use puffin::profile_scope;

use super::device::Device;
use super::{VulkanError, VulkanResult};

pub struct ShaderModule {
    pub(crate) bytes: Vec<u8>,
    pub(crate) raw: vk::ShaderModule,
}

impl ShaderModule {
    #[allow(dead_code)]
    pub fn destroy_shader(&self, device: &Device) {
        unsafe {
            device.destroy_shader_module(self.raw, None);
        }
    }
}

pub struct ShaderBuilder<'a> {
    device: &'a Device,
    bytecode: Option<&'a [u32]>,
}

impl<'a> ShaderBuilder<'a> {
    pub fn new(device: &'a Device) -> Self {
        Self {
            device,
            bytecode: None
        }
    }

    pub fn bytecode(mut self, bytecode: &'a [u32]) -> Self {
        self.bytecode = Some(bytecode);
        self
    }

    pub fn build(self) -> VulkanResult<ShaderModule> {
        profile_scope!("ShaderModule");

        let device = self.device;
        let code = self.bytecode.unwrap();

        let create_info = vk::ShaderModuleCreateInfo::default().code(&code);

        let shader = unsafe {
            device
                .create_shader_module(&create_info, None)
                .map_err(|e| VulkanError::Unknown(e))?
        };

        Ok(ShaderModule {
            raw: shader,
            bytes: bytemuck::cast_slice(&code).to_vec()
        })
    }
}

pub(crate) fn read_shader_from_bytes(bytes: &[u8]) -> Result<Vec<u32>, Box<dyn Error>> {
    let mut cursor = std::io::Cursor::new(bytes);
    Ok(ash::util::read_spv(&mut cursor)?)
}

pub(crate) fn load_spv<T: AsRef<Path>>(path: T) -> Vec<u32> {

    let mut file = std::fs::File::open(path).unwrap();
    let mut text = Vec::new();
    file.read_to_end(&mut text).unwrap();

    assert_eq!(text.len() % 4, 0);
    assert_eq!(
        0x07230203,
        u32::from_le_bytes([text[0], text[1], text[2], text[3]])
    );

    read_shader_from_bytes(&text).unwrap()
}
