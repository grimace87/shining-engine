
use crate::mem::{
    MemoryAllocator, ManagesImageMemory, MemoryAllocation
};
use crate::VkError;

use ash::vk;

impl ManagesImageMemory for MemoryAllocator {

    unsafe fn create_image(
        &self,
        image_info: &vk::ImageCreateInfo,
        for_staging: bool
    ) -> Result<(vk::Image, MemoryAllocation), VkError> {

        let image = self.device.create_image(&image_info, None)
            .map_err(|e| {
                VkError::OpFailed(format!("Error creating image: {:?}", e))
            })?;

        let memory_type = match for_staging {
            true => self.allocation_parameters
                .memory_type_staging_buffer
                .unwrap_or_else(|| self.allocation_parameters.memory_type_bulk_performance),
            false => self.allocation_parameters.memory_type_bulk_performance
        };
        let requirements = self.device.get_image_memory_requirements(image);
        let allocate_info = vk::MemoryAllocateInfo::builder()
            .allocation_size(requirements.size)
            .memory_type_index(memory_type);
        let memory = self.device.allocate_memory(&allocate_info, None)
            .map_err(|e| {
                VkError::OpFailed(format!("Error allocating image memory: {:?}", e))
            })?;
        let allocation = MemoryAllocation {
            memory,
            size: requirements.size
        };
        Ok((image, allocation))
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
