use std::ffi::CStr;

use ash::vk;
use log::{debug, info};
use winit::raw_window_handle::HasDisplayHandle;

use crate::core::Extension;

use super::app::App;
use super::debug::DebugCallback;
use super::{VulkanError, VulkanResult};

/// Unsafe Wrapper around [`vk::Instance`]
/// Required manually destroy before Drop
pub struct Instance {
    pub(crate) raw: ash::Instance,
    pub(crate) extensions: Vec<Extension>,
    pub debug_callback: Option<DebugCallback>,
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

impl Instance {

    pub fn new(window: &winit::window::Window, app: &App) -> VulkanResult<Instance> {

        let available_extensions = unsafe {
            app
                .entry
                .enumerate_instance_extension_properties(None)
                .map_err(VulkanError::Unknown)
        }?;

        let mut available_extension_names = vec![];

        for i in &available_extensions {
            if let Ok(name) = i.extension_name_as_c_str() {
                available_extension_names.push(name);
            }
        }

        let available_layers = unsafe {
            app
                .entry
                .enumerate_instance_layer_properties()
                .map_err(VulkanError::Unknown)
        }?;

        let mut available_layer_names = vec![];

        for i in &available_layers {
            if let Ok(name) = i.layer_name_as_c_str() {
                available_layer_names.push(name);
            }
        }

        let mut layers = vec![];

        #[cfg(feature = "validation_layer")]
        if available_layer_names.contains(&c"VK_LAYER_KHRONOS_validation") {
            layers.push(c"VK_LAYER_KHRONOS_validation");
        }

        let window_extensions = ash_window::enumerate_required_extensions(window.display_handle().unwrap().into())
            .map_err(VulkanError::Unknown)?
            .iter()
            .map(|ptr| unsafe { CStr::from_ptr(*ptr) })
            .collect::<Vec<_>>();

        let mut extensions = vec![];

        #[cfg(feature = "validation_layer")]
        if available_extension_names.contains(&ash::ext::debug_utils::NAME) {
            extensions.push(ash::ext::debug_utils::NAME);
        }

        extensions.extend(window_extensions);

        let p_extensions = extensions
            .iter()
            .map(|name| (*name).as_ptr())
            .collect::<Vec<_>>();

        let p_layers = layers
            .iter()
            .map(|name| (*name).as_ptr())
            .collect::<Vec<_>>();

        let create_info = vk::InstanceCreateInfo::default()
            .enabled_layer_names(&p_layers)
            .enabled_extension_names(&p_extensions)
            .application_info(&app.create_info);

        let instance = unsafe {
            app
                .entry
                .create_instance(&create_info, None)
                .map_err(VulkanError::Unknown)
        }?;

        let debug_callback = if extensions.contains(&ash::ext::debug_utils::NAME) && layers.contains(&c"VK_LAYER_KHRONOS_validation") {
            Some(DebugCallback::new(&app.entry, &instance))
        } else {
            None
        };

        Ok(Instance {
            raw: instance,
            extensions: vec![],
            debug_callback,
        })
    }
}
