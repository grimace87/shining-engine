
mod instance;
mod debug;
mod physical_device;

use crate::VkError;
use physical_device::Queues;
use ash::{
    Entry,
    Instance,
    extensions::{
        ext::DebugUtils,
        khr::Surface
    },
    version::InstanceV1_0,
    vk
};
use raw_window_handle::HasRawWindowHandle;

/// FeatureDeclaration enum
/// Platform feature requirements that may be declared by an application or component thereof in
/// advance, in case it's needed during initialisation.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum FeatureDeclaration {
    ClipPlanes // Vulkan - see VkPhysicalDeviceFeatures.shaderClipDistance
}

/// Wrap Vulkan components that can exist for the life of the app once successfully created
pub struct VkCore {
    pub function_loader: Entry,
    pub instance: Instance,
    debug_utils: Option<(DebugUtils, vk::DebugUtilsMessengerEXT)>,
    pub physical_device: vk::PhysicalDevice,
    pub queues: Queues,
    pub physical_device_features: vk::PhysicalDeviceFeatures
}

impl VkCore {

    pub unsafe fn new(
        window_owner: &dyn HasRawWindowHandle,
        features: Vec<FeatureDeclaration>
    ) -> Result<Self, VkError> {

        let entry = Entry::new()
            .map_err(|e| {
                VkError::OpFailed(format!("Entry creation failed: {:?}", e))
            })?;

        let instance = instance::make_instance(&entry, window_owner)?;
        let debug_utils = debug::make_debug_utils(&entry, &instance)?;

        // Create temporary surface and surface loader
        let surface_fn = Surface::new(&entry, &instance);
        let surface = ash_window::create_surface(
            &entry,
            &instance,
            window_owner,
            None)
            .unwrap();

        // Now select a physical device
        let (physical_device, queues, physical_device_features) =
            physical_device::select_physical_device(
                &instance,
                &surface_fn,
                &surface,
                &features)?;

        // Destroy the temporary surface
        surface_fn.destroy_surface(surface, None);

        Ok(Self {
            function_loader: entry,
            instance,
            debug_utils,
            physical_device,
            queues,
            physical_device_features
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
