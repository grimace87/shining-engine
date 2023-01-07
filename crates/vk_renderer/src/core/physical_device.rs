
use crate::core::FeatureDeclaration;
use crate::VkError;
use ash::{vk, extensions::khr::Surface};

/// Selects the physical device to use, so long as there is one that supports everything needed
pub unsafe fn select_physical_device(
    instance: &ash::Instance,
    surface_loader: &Surface,
    surface: &vk::SurfaceKHR,
    features: &[FeatureDeclaration]
) -> Result<(vk::PhysicalDevice, u32, u32, vk::PhysicalDeviceFeatures), VkError> {

    let physical_devices = instance
        .enumerate_physical_devices()
        .map_err(|e| {
            VkError::OpFailed(format!("{:?}", e))
        })?;
    if physical_devices.is_empty() {
        return Err(VkError::OpFailed(
            String::from("No physical devices found")));
    }

    let unset_value: u32 = u32::MAX;
    for physical_device in physical_devices.iter() {
        let queue_family_properties =
            instance.get_physical_device_queue_family_properties(*physical_device);
        let mut graphics_index: u32 = unset_value;
        let mut transfer_index: u32 = unset_value;
        let mut features_to_enable = vk::PhysicalDeviceFeatures::default();
        for (index, properties) in queue_family_properties.iter().enumerate() {

            let supports_graphics =
                properties.queue_flags.contains(vk::QueueFlags::GRAPHICS);
            let supports_surface = surface_loader
                .get_physical_device_surface_support(
                    *physical_device,
                    index as u32,
                    *surface)
                .unwrap();
            let supports_transfer =
                properties.queue_flags.contains(vk::QueueFlags::TRANSFER);

            let supported_features =
                instance.get_physical_device_features(*physical_device);
            features_to_enable = match make_feature_set_to_enable(features, &supported_features) {
                Some(features) => features,
                None => continue
            };

            let graphics_and_surface = supports_graphics && supports_surface;
            if graphics_and_surface {
                graphics_index = index as u32;
            }
            if supports_transfer && (transfer_index == unset_value || !graphics_and_surface) {
                transfer_index = index as u32;
            }
        }
        if graphics_index != unset_value && transfer_index != unset_value {
            return Ok((
                *physical_device,
                graphics_index,
                transfer_index,
                features_to_enable
            ));
        }
    }

    Err(VkError::OpFailed(
        String::from("Could not find a suitable physical device")))
}

/// Return set of features to enable during device creation, knowing that all of those features
/// are supported by the physical device. If they are not all supported, this returns None.
fn make_feature_set_to_enable(
    features: &[FeatureDeclaration],
    supported_features: &vk::PhysicalDeviceFeatures
) -> Option<vk::PhysicalDeviceFeatures> {
    let mut features_to_enable = vk::PhysicalDeviceFeatures::default();
    for feature in features.iter() {
        match feature {
            FeatureDeclaration::ClipPlanes => {
                if supported_features.shader_clip_distance == vk::TRUE {
                    features_to_enable.shader_clip_distance = vk::TRUE;
                } else {
                    return None;
                }
            }
        }
    }
    Some(features_to_enable)
}
