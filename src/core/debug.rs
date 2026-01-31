use ash::vk;

pub struct DebugCallback {
    callback: vk::DebugUtilsMessengerEXT,
    loader: ash::ext::debug_utils::Instance,
}

impl DebugCallback {
    pub fn destroy(&self) {
        unsafe {
            self.loader
                .destroy_debug_utils_messenger(self.callback, None)
        };
    }

    pub fn new(entry: &ash::Entry, instance: &ash::Instance) -> Self {
        let debug_info = vk::DebugUtilsMessengerCreateInfoEXT::default()
            .message_severity(
                vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE
                    | vk::DebugUtilsMessageSeverityFlagsEXT::INFO
                    | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                    | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
            )
            .message_type(
                vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                    | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                    | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
            )
            .pfn_user_callback(Some(vulkan_debug_callback));

        let loader = ash::ext::debug_utils::Instance::new(entry, instance);

        let callback = unsafe {
            loader
                .create_debug_utils_messenger(&debug_info, None)
                .unwrap()
        };

        DebugCallback { callback, loader }
    }
}

pub unsafe extern "system" fn vulkan_debug_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT<'_>,
    _user_data: *mut std::os::raw::c_void,
) -> vk::Bool32 {
    let callback_data = unsafe { *p_callback_data };

    if callback_data.p_message.is_null() {
        return vk::FALSE;
    }

    let message = unsafe { std::ffi::CStr::from_ptr(callback_data.p_message).to_string_lossy() };

    if message_severity == vk::DebugUtilsMessageSeverityFlagsEXT::WARNING {
        log::warn!("{}", message);
    }

    if message_severity == vk::DebugUtilsMessageSeverityFlagsEXT::ERROR {
        log::error!("{}", message)
    }

    if message_severity == vk::DebugUtilsMessageSeverityFlagsEXT::INFO {
        // tracing::info!("{}", message)
    }

    vk::FALSE
}
