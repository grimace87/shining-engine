
use crate::VkError;
use crate::mem::{MemoryUsage, MemoryAllocator, MemoryAllocationCreateInfo, MemoryAllocation};
use ash::vk;

/// BufferWrapper struct
/// Wraps up a Vulkan Buffer and its memory allocation that backs it
pub struct BufferWrapper {
    pub buffer: vk::Buffer,
    allocation: MemoryAllocation
}

impl BufferWrapper {

    /// Create a new buffer and back it with memory
    pub unsafe fn new(
        allocator: &MemoryAllocator,
        size_bytes: usize,
        buffer_usage: vk::BufferUsageFlags,
        mem_usage: MemoryUsage
    ) -> Result<BufferWrapper, VkError> {
        let buffer_create_info = vk::BufferCreateInfo::builder()
            .size(size_bytes as u64)
            .usage(buffer_usage)
            .build();
        let memory_create_info = MemoryAllocationCreateInfo {
            usage: mem_usage
        };
        let (buffer, allocation) = allocator
            .create_buffer(&buffer_create_info, &memory_create_info)?;

        Ok(BufferWrapper {
            buffer,
            allocation
        })
    }

    /// Return a new instance, with no buffer or memory associated with it
    pub fn empty() -> BufferWrapper {
        BufferWrapper {
            buffer: vk::Buffer::null(),
            allocation: MemoryAllocation::null()
        }
    }

    /// Clean up the contained resources
    pub unsafe fn destroy(&self, allocator: &MemoryAllocator) -> Result<(), VkError> {
        allocator.destroy_buffer(self.buffer, &self.allocation)
            .map_err(|e| {
                VkError::OpFailed(format!("Error freeing buffer: {:?}", e))
            })
    }

    /// Map the backed memory, then update it from a host-owned pointer
    pub unsafe fn update<T: Sized>(
        &mut self,
        allocator: &MemoryAllocator,
        dst_offset_elements: isize,
        src_ptr: *const T,
        element_count: usize
    ) -> Result<(), VkError> {
        let mut dst_ptr = allocator.map_memory::<T>(&self.allocation)?;
        dst_ptr = dst_ptr.offset(dst_offset_elements);
        dst_ptr.copy_from_nonoverlapping(src_ptr, element_count);
        allocator.unmap_memory(&self.allocation).unwrap();
        Ok(())
    }

    /// Getter for the buffer within
    pub fn buffer(&self) -> vk::Buffer {
        self.buffer
    }
}
