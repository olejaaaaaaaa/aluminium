use std::ffi::CStr;

use ash::vk;
use log::{debug, info};

use super::{VulkanError, VulkanResult};

const MAX_VK_API_VERSION: u32 = vk::API_VERSION_1_0;
const ENGINE_VERSION: u32 = 0;
const ENGINE_NAME: &CStr = c"Aluminium";
const APP_NAME: &CStr = c"App";
const APP_VERSION: u32 = 0;

pub struct App {
    pub(crate) create_info: vk::ApplicationInfo<'static>,
    pub(crate) entry: ash::Entry,
}

impl App {

    pub fn new() -> VulkanResult<App> {

        let entry = unsafe {
            ash::Entry::load()
                .map_err(|e| VulkanError::App(crate::core::errors::app::AppError::LoadingVulkan(e)))
        }?;

        let create_info = vk::ApplicationInfo::default()
            .api_version(MAX_VK_API_VERSION)
            .application_version(APP_VERSION)
            .application_name(APP_NAME)
            .engine_name(ENGINE_NAME)
            .engine_version(ENGINE_VERSION);

        info!("VK_API_VERSION: {:?}", MAX_VK_API_VERSION);

        Ok(App { create_info, entry })
    }
}
