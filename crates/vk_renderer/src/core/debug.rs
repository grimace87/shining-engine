
use crate::VkError;
use ash::{
    vk,
    Entry,
    Instance,
    extensions::ext::DebugUtils
};
use std::ffi::CStr;

/// Simple debug logger; calls println to display message with type and severity
unsafe extern "system" fn vulkan_debug_utils_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _p_user_data: *mut std::ffi::c_void
) -> vk::Bool32 {
    let message = CStr::from_ptr((*p_callback_data).p_message);
    let severity = format!("{:?}", message_severity);
    let ty = format!("{:?}", message_type);
    println!("[Debug][{}][{}] {:?}", severity, ty, message);
    vk::FALSE
}

/// Construct a debug messenger; it will be in effect immediately
pub unsafe fn make_debug_utils(
    entry: &Entry,
    instance: &Instance
) -> Result<Option<(DebugUtils, vk::DebugUtilsMessengerEXT)>, VkError> {
    if cfg!(debug_assertions) {
        let debug_utils = DebugUtils::new(entry, instance);
        let debug_create_info = vk::DebugUtilsMessengerCreateInfoEXT {
            message_severity: vk::DebugUtilsMessageSeverityFlagsEXT::WARNING |
                vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
            message_type: vk::DebugUtilsMessageTypeFlagsEXT::GENERAL |
                vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE |
                vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION,
            pfn_user_callback: Some(vulkan_debug_utils_callback),
            ..Default::default()
        };
        let utils_messenger = debug_utils
            .create_debug_utils_messenger(&debug_create_info, None)
            .map_err(|e| {
                VkError::OpFailed(format!("Debug messenger creation failed: {:?}", e))
            })?;
        Ok(Some((debug_utils, utils_messenger)))
    } else {
        Ok(None)
    }
}
