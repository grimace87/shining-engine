
use crate::VkContext;
use crate::mem::{MemoryAllocator, MemoryAllocation, ManagesBufferMemory};
use ecs::{EcsManager, resource::Resource};
use error::EngineError;
use ash::vk;

/// ImageUsage enum
/// An enumeration of what purpose buffer resources can be used for
#[derive(PartialEq, Copy, Clone, Debug)]
pub enum BufferUsage {
    InitialiseOnceVertexBuffer,
    UniformBuffer
}

/// BufferCreationParams struct
/// Description for creating an buffer; should cover all use cases needed by the engine
struct BufferCreationParams {
    usage_flags: vk::BufferUsageFlags,
    host_accessible: bool
}

/// BufferWrapper struct
/// Wraps up a Vulkan Buffer and its memory allocation that backs it
pub struct BufferWrapper {
    pub buffer: vk::Buffer,
    pub size_bytes: usize,
    pub element_count: usize,
    allocation: MemoryAllocation
}

/// VboCreationData struct
/// Specification for how a vertex buffer is to be created
pub struct VboCreationData {
    pub vertex_data: Option<*const u8>,
    pub vertex_size_bytes: usize,
    pub vertex_count: usize,
    pub draw_indexed: bool,
    pub index_data: Option<Vec<u16>>,
    pub usage: BufferUsage
}

impl Resource<VkContext> for BufferWrapper {
    type CreationData = VboCreationData;

    fn create(
        loader: &VkContext,
        _ecs: &EcsManager<VkContext>,
        data: &VboCreationData
    ) -> Result<Self, EngineError> {
        let buffer = unsafe {
            BufferWrapper::new(
                loader,
                data.usage,
                data.vertex_count * data.vertex_size_bytes,
                data.vertex_count,
                data.vertex_data)?
        };
        Ok(buffer)
    }

    fn release(&self, loader: &VkContext) {
        let (allocator, _) = loader.get_mem_allocator();
        unsafe {
            allocator.destroy_buffer(self.buffer, &self.allocation)
                .map_err(|e| {
                    EngineError::OpFailed(format!("Error freeing buffer: {:?}", e))
                })
                .unwrap();
        }
    }
}

impl BufferWrapper {

    /// Create a new buffer and back it with memory
    unsafe fn new(
        context: &VkContext,
        buffer_usage: BufferUsage,
        size_bytes: usize,
        element_count: usize,
        init_data: Option<*const u8>
    ) -> Result<BufferWrapper, EngineError> {

        let transfer_usage = match init_data.is_some() {
            true => vk::BufferUsageFlags::TRANSFER_DST,
            false => vk::BufferUsageFlags::empty()
        };

        let creation_params = match buffer_usage {
            BufferUsage::InitialiseOnceVertexBuffer => BufferCreationParams {
                usage_flags: vk::BufferUsageFlags::VERTEX_BUFFER | transfer_usage,
                host_accessible: false
            },
            BufferUsage::UniformBuffer => BufferCreationParams {
                usage_flags: vk::BufferUsageFlags::UNIFORM_BUFFER | transfer_usage,
                host_accessible: true
            }
        };

        let buffer_create_info = vk::BufferCreateInfo::builder()
            .size(size_bytes as u64)
            .usage(creation_params.usage_flags)
            .build();
        let buffer = context.device.create_buffer(&buffer_create_info, None)
            .map_err(|e| {
                EngineError::OpFailed(format!("Error creating buffer: {:?}", e))
            })?;

        let (allocator, transfer_queue) = context.get_mem_allocator();
        let allocation = allocator.back_buffer_memory(
            transfer_queue,
            &buffer,
            creation_params.host_accessible,
            init_data,
            size_bytes)?;

        Ok(BufferWrapper {
            buffer,
            size_bytes,
            element_count,
            allocation
        })
    }

    /// Return a new instance, with no buffer or memory associated with it
    pub fn empty() -> BufferWrapper {
        BufferWrapper {
            buffer: vk::Buffer::null(),
            size_bytes: 0,
            element_count: 0,
            allocation: MemoryAllocation::null()
        }
    }

    /// Map the backed memory, then update it from a host-owned pointer
    pub unsafe fn update<T: Sized>(
        &self,
        allocator: &MemoryAllocator,
        dst_offset_elements: isize,
        src_ptr: *const T,
        element_count: usize
    ) -> Result<(), EngineError> {
        let offset_bytes = dst_offset_elements as usize * std::mem::size_of::<T>();
        let update_range_bytes = element_count * std::mem::size_of::<T>();
        if offset_bytes + update_range_bytes > self.size_bytes {
            return Err(EngineError::EngineError(format!(
                "Attempting to update buffer outside of range: offset {}, range {}, size {}",
                offset_bytes,
                update_range_bytes,
                self.size_bytes)))
        }
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
