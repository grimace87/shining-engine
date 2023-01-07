
use crate::mem::{MemoryAllocator, ManagesImageMemory, MemoryAllocation, ManagesMemoryTransfers};
use crate::{VkError, Queue};

use ash::vk;

impl ManagesImageMemory for MemoryAllocator {

    /// Prepares the image and its memory ready for its intended usage.
    /// After this function returns, the image will be backed by memory, and will be in the desired
    /// layout ready for use. If applicable, the bound memory will be initialised with the provided
    /// data.
    ///
    /// Three cases must be handled independently here:
    /// - No initialisation data; allocate memory of the optimal type and bind it to the image
    /// - Initialisation data, and no staging buffer needed (all device memory is host-visible);
    ///     allocate memory of the optimal type and copy initialisation data to it
    /// - Initialisation data, and staging buffer needed; use a staging buffer for copying
    ///     initialisation data
    unsafe fn back_image_memory(
        &self,
        transfer_queue: &Queue,
        image: &vk::Image,
        aspect: vk::ImageAspectFlags,
        width: u32,
        height: u32,
        init_layer_data: Option<&[Vec<u8>]>,
        initialising_layout: vk::ImageLayout,
        expected_layout: vk::ImageLayout
    ) -> Result<MemoryAllocation, VkError> {

        // Allocate the final memory to be used for backing the image
        let requirements = self.device.get_image_memory_requirements(*image);
        let allocate_info = vk::MemoryAllocateInfo::builder()
            .allocation_size(requirements.size)
            .memory_type_index(self.allocation_parameters.memory_type_bulk_performance);
        let memory = self.device.allocate_memory(&allocate_info, None)
            .map_err(|e| {
                VkError::OpFailed(format!("Error allocating image memory: {:?}", e))
            })?;
        let allocation = MemoryAllocation {
            memory,
            size: requirements.size
        };

        // Bind the image's memory
        self.device.bind_image_memory(*image, memory, 0)
            .map_err(|e| {
                VkError::OpFailed(format! ("Error binding memory to image: {:?}", e))
            })?;

        // If memory needs to be initialised with data, do it via a separate function that handles
        // the staging buffer (or doesn't use it if it's not applicable on this device). If no
        // data initialisation is needed, just transition the image to the layout ready for use.
        if let Some(layer_data) = init_layer_data {
            self.transfer_data_to_new_texture(
                transfer_queue,
                width,
                height,
                image,
                aspect,
                expected_layout,
                &allocation,
                layer_data)?;
        } else {
            self.transition_image_layout(
                transfer_queue,
                image,
                aspect,
                initialising_layout,
                expected_layout)?;
        }

        Ok(allocation)
    }

    unsafe fn destroy_image(
        &self,
        image: vk::Image,
        allocation: &MemoryAllocation
    ) -> Result<(), VkError> {
        self.device.destroy_image(image, None);
        self.device.free_memory(allocation.memory, None);
        Ok(())
    }
}
