
use crate::mem::{
    MemoryAllocator, ManagesBufferMemory, MemoryAllocation
};
use crate::VkError;

use ash::vk;

impl ManagesBufferMemory for MemoryAllocator {

    unsafe fn create_buffer(
        &self,
        buffer_info: &vk::BufferCreateInfo
    ) -> Result<(vk::Buffer, MemoryAllocation), VkError> {
        let buffer = self.device.create_buffer(&buffer_info, None)
            .map_err(|e| {
                VkError::OpFailed(format!("Error creating buffer: {:?}", e))
            })?;

        let requirements = self.device.get_buffer_memory_requirements(buffer);
        let allocate_info = vk::MemoryAllocateInfo::builder()
            .allocation_size(requirements.size);
        let memory = self.device.allocate_memory(&allocate_info, None)
            .map_err(|e| {
                VkError::OpFailed(format!("Error allocating buffer memory: {:?}", e))
            })?;
        let allocation = MemoryAllocation {
            memory,
            size: requirements.size
        };
        Ok((buffer, allocation))
    }

    unsafe fn destroy_buffer(
        &self,
        buffer: vk::Buffer,
        allocation: &MemoryAllocation
    ) -> Result<(), VkError> {
        self.device.destroy_buffer(buffer, None);
        self.device.free_memory(allocation.memory, None);
        Ok(())
    }
}
