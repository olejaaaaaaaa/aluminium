use std::ffi::CStr;

use ash::vk;
use log::debug;

use super::{VulkanError, VulkanResult};

const ENGINE_VERSION: u32 = 0;
const ENGINE_NAME: &'static CStr = c"Aluminium";

pub struct App {
    pub(crate) create_info: vk::ApplicationInfo<'static>,
    pub(crate) entry: ash::Entry,
}

pub struct AppBuilder {
    max_api_version: u32,
    min_api_version: u32,
    app_name: &'static CStr,
    app_version: u32,
}

impl AppBuilder {
    /// Default App
    ///
    /// ```
    /// AppBuilder {
    ///     app_name: c"App",
    ///     app_version: 0,
    ///     max_api_version: vk::API_VERSION_1_0,
    ///     min_api_version: vk::API_VERSION_1_0,
    /// }
    /// ```
    pub fn default() -> Self {
        AppBuilder {
            app_name: c"App",
            app_version: 0,
            max_api_version: vk::API_VERSION_1_0,
            min_api_version: vk::API_VERSION_1_0,
        }
    }

    // The highest supported api version if available
    pub fn with_max_api_version(mut self, version: u32) -> Self {
        self.max_api_version = version;
        self
    }

    // Minimum required api version support
    pub fn with_min_api_version(mut self, version: u32) -> Self {
        self.min_api_version = version;
        self
    }

    /// Set App name
    pub fn with_app_name(mut self, name: &'static CStr) -> Self {
        self.app_name = name;
        self
    }

    pub fn build(self) -> VulkanResult<App> {
        let entry = unsafe {
            ash::Entry::load()
                .map_err(|e| VulkanError::App(crate::core::errors::app::AppError::LoadingVulkan(e)))
        }?;

        let available_api_version = unsafe {
            entry
                .try_enumerate_instance_version()
                .map_err(|e| {
                    VulkanError::App(crate::core::errors::app::AppError::LoadingVulkanApiVersion(
                        e,
                    ))
                })?
                .unwrap_or(vk::API_VERSION_1_0)
        };

        if self.min_api_version > available_api_version {
            return Err(VulkanError::App(crate::core::errors::app::AppError::Api(
                available_api_version,
            )));
        }

        let api_version = if self.max_api_version > available_api_version {
            available_api_version
        } else {
            self.max_api_version
        };

        let create_info = vk::ApplicationInfo::default()
            .api_version(api_version)
            .application_version(self.app_version)
            .application_name(self.app_name)
            .engine_name(ENGINE_NAME)
            .engine_version(ENGINE_VERSION);

        debug!("App: {:?}", create_info);

        Ok(App { create_info, entry })
    }
}

#[test]
fn app() {
    let app = AppBuilder::default().build();
    assert!(app.is_ok());
}
