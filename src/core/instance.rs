use std::ffi::CStr;

use ash::vk;
use log::debug;

use super::app::App;
use super::debug::DebugCallback;
use super::{VulkanError, VulkanResult};

/// Unsafe Wrapper around [`vk::Instance`]
/// Required manually destroy before Drop
pub struct Instance {
    pub(crate) raw: ash::Instance,
    #[allow(dead_code)]
    pub(crate) layers: Vec<&'static CStr>,
    #[allow(dead_code)]
    pub(crate) extensions: Vec<&'static CStr>,
    pub(crate) debug_callback: Option<DebugCallback>,
}

impl Instance {
    /// Safety if all child object destroyed before
    pub fn destroy(&self) {
        if let Some(debug) = &self.debug_callback {
            debug.destroy();
        }
        unsafe { self.raw.destroy_instance(None) };
    }
}

pub struct InstanceBuilder<'a> {
    app: &'a App,
    enable_debug: bool,
}

impl<'a> InstanceBuilder<'a> {
    pub fn default(app: &'a App) -> Self {
        Self {
            app,
            enable_debug: true,
        }
    }

    pub fn build(self) -> VulkanResult<Instance> {
        let _extensions = unsafe {
            self.app
                .entry
                .enumerate_instance_extension_properties(None)
                .map_err(|e| VulkanError::Unknown(e))
        }?;

        let layers = [c"VK_LAYER_KHRONOS_validation"];

        let extenions = [
            c"VK_KHR_win32_surface",
            c"VK_EXT_debug_utils",
            c"VK_KHR_surface",
        ];

        let p_extenions = extenions
            .iter()
            .map(|name| (*name).as_ptr())
            .collect::<Vec<_>>();

        let p_layers = layers
            .iter()
            .map(|name| (*name).as_ptr())
            .collect::<Vec<_>>();

        let create_info = vk::InstanceCreateInfo::default()
            .enabled_layer_names(&p_layers)
            .enabled_extension_names(&p_extenions)
            .application_info(&self.app.create_info);

        debug!("Instance: {:?}", create_info);

        let instance = unsafe {
            self.app
                .entry
                .create_instance(&create_info, None)
                .map_err(|e| VulkanError::Unknown(e))
        }?;

        let debug = if self.enable_debug {
            Some(DebugCallback::new(&self.app.entry, &instance))
        } else {
            None
        };

        Ok(Instance {
            raw: instance,
            layers: vec![],
            extensions: vec![],
            debug_callback: debug,
        })
    }
}
