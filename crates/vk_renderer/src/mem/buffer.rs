
use crate::mem::{MemoryAllocator, ManagesBufferMemory, MemoryAllocation, ManagesMemoryTransfers};
use crate::{VkError, Queue};

use ash::vk;

impl ManagesBufferMemory for MemoryAllocator {

    /// Prepares the buffer and its memory ready for its intended usage.
    /// After this function returns, the buffer will be backed by memory, and that memory will be
    /// initialised with data if some was provided. If requested, the memory will be host-visible.
    unsafe fn back_buffer_memory(
        &self,
        transfer_queue: &Queue,
        buffer: &vk::Buffer,
        host_accessible: bool,
        init_data: Option<*const u8>,
        init_data_size_bytes: usize
    ) -> Result<MemoryAllocation, VkError> {

        // Allocate the final memory to be used for backing the buffer
        let requirements = self.device.get_buffer_memory_requirements(*buffer);
        let memory_type = match host_accessible {
            true => self.allocation_parameters.memory_type_host_visible,
            false => self.allocation_parameters.memory_type_bulk_performance
        };
        let allocate_info = vk::MemoryAllocateInfo::builder()
            .allocation_size(requirements.size)
            .memory_type_index(memory_type);
        let memory = self.device.allocate_memory(&allocate_info, None)
            .map_err(|e| {
                VkError::OpFailed(format!("Error allocating buffer memory: {:?}", e))
            })?;
        let allocation = MemoryAllocation {
            memory,
            size: requirements.size
        };

        // Bind the buffer's memory
        self.device.bind_buffer_memory(*buffer, memory, 0)
            .map_err(|e| {
                VkError::OpFailed(format! ("Error binding memory to image: {:?}", e))
            })?;

        // If memory needs to be initialised with data, do it via a separate function that handles
        // the staging buffer (or doesn't use it if it's not applicable on this device).
        if let Some(data) = init_data {
            self.transfer_data_to_new_buffer(
                transfer_queue,
                buffer,
                &allocation,
                data,
                init_data_size_bytes)?;
        }

        Ok(allocation)
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
