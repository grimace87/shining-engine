
use error::EngineError;
use ash::{
    vk,
    Entry,
    Instance,
    extensions::ext::DebugUtils
};
use raw_window_handle::RawDisplayHandle;
use std::{
    ffi::{
        CString,
        CStr
    },
    os::raw::c_char
};

const DEBUG_LAYER_NAME: &'static str = "VK_LAYER_KHRONOS_validation";

/// Creates the instance, enabling any required extensions and layers
pub unsafe fn make_instance(
    entry: &Entry,
    display_handle: RawDisplayHandle
) -> Result<Instance, EngineError> {

    // App info
    let engine_name = CString::new("Shining Engine").unwrap();
    let app_name = CString::new("Shining Engine Sample").unwrap();
    let app_info = vk::ApplicationInfo::builder()
        .application_name(&app_name)
        .application_version(vk::make_api_version(0, 0, 1, 0))
        .engine_name(&engine_name)
        .engine_version(vk::make_api_version(0, 0, 0, 1))
        .api_version(vk::make_api_version(0, 1, 0, 0));

    // Instance extensions and validation layers
    let mut instance_extensions = get_debug_instance_extensions(entry)?;
    let required_platform_extensions = get_window_instance_extensions(display_handle)?;
    instance_extensions.extend(&required_platform_extensions);

    // Validation layers
    let debug_layers = get_debug_instance_layers(entry)?;
    let layer_name_pointers: Vec<_> = debug_layers
        .iter()
        .map(|name| name.as_ptr())
        .collect();

    // Create the instance
    let instance_create_info = vk::InstanceCreateInfo::builder()
        .application_info(&app_info)
        .enabled_extension_names(&instance_extensions)
        .enabled_layer_names(&layer_name_pointers);
    entry
        .create_instance(&instance_create_info, None)
        .map_err(|e| {
            EngineError::OpFailed(format!("Instance creation failed: {:?}", e))
        })
}

/// Get the required extensions for windowing - this will be handled by ash_window
fn get_window_instance_extensions(
    display_handle: RawDisplayHandle
) -> Result<Vec<*const c_char>, EngineError> {
    let extensions_as_c_str =
        ash_window::enumerate_required_extensions(display_handle)
            .map_err(|e| {
                EngineError::OpFailed(format!("{:?}", e))
            })?
            .iter()
            .map(|ext| *ext)
            .collect::<Vec<*const c_char>>();
    Ok(extensions_as_c_str)
}

/// Gets the extensions required for debugging
unsafe fn get_debug_instance_extensions(entry: &Entry) -> Result<Vec<*const c_char>, EngineError> {
    if cfg!(debug_assertions) {
        let debug_extension = DebugUtils::name();
        let supported_extensions = entry.enumerate_instance_extension_properties(None)
            .map_err(|e| {
                EngineError::OpFailed(format!("Failed to enumerate instance extensions: {:?}", e))
            })?;
        let is_supported = supported_extensions
            .iter()
            .any(|ext| CStr::from_ptr(ext.extension_name.as_ptr()).eq(debug_extension));
        if is_supported {
            Ok(vec![DebugUtils::name().as_ptr()])
        } else {
            Ok(vec![])
        }
    } else {
        Ok(vec![])
    }
}

/// Gets the instance layers for debugging
unsafe fn get_debug_instance_layers(entry: &Entry) -> Result<Vec<CString>, EngineError> {
    if cfg!(debug_assertions) {
        let validation_layer = CString::new(DEBUG_LAYER_NAME).unwrap();
        let supported_extensions = entry.enumerate_instance_layer_properties()
            .map_err(|e| {
                EngineError::OpFailed(format!("Failed to enumerate instance layers: {:?}", e))
            })?;
        let is_supported = supported_extensions
            .iter()
            .any(|layer| {
                validation_layer.as_c_str().eq(CStr::from_ptr(layer.layer_name.as_ptr()))
            });
        if is_supported {
            Ok(vec![validation_layer])
        } else {
            Ok(vec![])
        }
    } else {
        Ok(vec![])
    }
}
