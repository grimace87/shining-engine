
mod instance;
mod debug;

use crate::VkError;
use ash::{
    Entry,
    Instance,
    extensions::ext::DebugUtils,
    version::InstanceV1_0,
    vk
};
use raw_window_handle::HasRawWindowHandle;

/// Wrap Vulkan components that can exist for the life of the app once successfully created
pub struct VkCore {
    function_loader: Entry,
    instance: Instance,
    debug_utils: Option<(DebugUtils, vk::DebugUtilsMessengerEXT)>
}

impl VkCore {

    pub unsafe fn new(window_source: &dyn HasRawWindowHandle) -> Result<Self, VkError> {

        let entry = Entry::new()
            .map_err(|e| {
                VkError::OpFailed(format!("Entry creation failed: {:?}", e))
            })?;

        let instance = instance::make_instance(&entry, window_source)?;
        let debug_utils = debug::make_debug_utils(&entry, &instance)?;

        Ok(Self {
            function_loader: entry,
            instance,
            debug_utils
        })
    }
}

impl Drop for VkCore {

    fn drop(&mut self) {
        unsafe {
            if let Some((debug_utils, utils_messenger)) = &self.debug_utils {
                debug_utils.destroy_debug_utils_messenger(*utils_messenger, None);
            }
            self.instance.destroy_instance(None);
        }
    }
}
