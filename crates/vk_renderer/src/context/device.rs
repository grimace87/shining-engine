
use crate::VkCore;
use error::EngineError;
use ash::{vk, Device, extensions::khr::{Swapchain}};
use std::os::raw::c_char;

/// All device-related initialisation - chooses a physical device, creates the logical device, and
/// creates a single graphics queue and single transfer queue
pub unsafe fn make_device_resources(
    core: &VkCore
) -> Result<Device, EngineError> {

    // Find queue indices for graphics and transfer (ideally different but could be the same)
    let queue_family_properties = core.instance
        .get_physical_device_queue_family_properties(core.physical_device);
    let (graphics_queue_family_index, transfer_queue_family_index) = {
        let mut found_graphics_queue_index = None;
        let mut found_transfer_queue_index = None;
        for (index, queue_family) in queue_family_properties.iter().enumerate() {
            let graphics_flag = queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS);
            if queue_family.queue_count > 0 && graphics_flag {
                found_graphics_queue_index = Some(index as u32);
            }
            let transfer_flag = queue_family.queue_flags.contains(vk::QueueFlags::TRANSFER);
            if queue_family.queue_count > 0 && transfer_flag {
                if found_transfer_queue_index.is_none() || !graphics_flag {
                    found_transfer_queue_index = Some(index as u32);
                }
            }
        }
        (
            found_graphics_queue_index.unwrap(),
            found_transfer_queue_index.unwrap()
        )
    };

    // Device extensions required
    let device_extensions: Vec<*const c_char> = vec![ Swapchain::name().as_ptr() ];

    // Make the logical device
    let priorities = [1.0f32];
    let queue_infos = [
        vk::DeviceQueueCreateInfo::builder()
            .queue_family_index(graphics_queue_family_index)
            .queue_priorities(&priorities)
            .build(),
        vk::DeviceQueueCreateInfo::builder()
            .queue_family_index(transfer_queue_family_index)
            .queue_priorities(&priorities)
            .build()
    ];
    let device_create_info = vk::DeviceCreateInfo::builder()
        .queue_create_infos(&queue_infos)
        .enabled_extension_names(&device_extensions)
        .enabled_features(&core.physical_device_features);
    let device = core.instance
        .create_device(
            core.physical_device,
            &device_create_info,
            None)
        .map_err(|e| {
            EngineError::OpFailed(format!("{:?}", e))
        })?;

    Ok(device)
}
