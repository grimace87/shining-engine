
use crate::mem::{
    MemoryAllocator, ManagesImageMemory, MemoryAllocation
};
use crate::VkError;

use ash::vk;

impl ManagesImageMemory for MemoryAllocator {

    unsafe fn create_image(
        &self,
        image_info: &vk::ImageCreateInfo
    ) -> Result<(vk::Image, MemoryAllocation), VkError> {

        let image = self.device.create_image(&image_info, None)
            .map_err(|e| {
                VkError::OpFailed(format!("Error creating image: {:?}", e))
            })?;

        let requirements = self.device.get_image_memory_requirements(image);
        let allocate_info = vk::MemoryAllocateInfo::builder()
            .allocation_size(requirements.size);
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
