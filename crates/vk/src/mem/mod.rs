
use crate::VkError;

use ash::{Device, vk};

pub enum MemoryUsage {
    CpuToGpu,
    GpuOnly
}

pub struct MemoryAllocatorCreateInfo {
    pub physical_device: vk::PhysicalDevice,
    pub device: Device
}

pub struct MemoryAllocator {
    physical_device: vk::PhysicalDevice,
    device: Device
}

impl MemoryAllocator {

    pub fn new(allocator_info: MemoryAllocatorCreateInfo) -> Self {

        // Decide how buffers and images should be allocated later
        // TODO let memory_heaps = allocator_info.device;

        Self {
            physical_device: allocator_info.physical_device,
            device: allocator_info.device
        }
    }

    pub fn destroy(&mut self) {

    }

    pub unsafe fn create_buffer(
        &self,
        buffer_info: &vk::BufferCreateInfo,
        memory_info: &MemoryAllocationCreateInfo
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
        // Ok((vk::Buffer::null(), MemoryAllocation {}))
    }

    pub unsafe fn destroy_buffer(
        &self,
        buffer: vk::Buffer,
        allocation: &MemoryAllocation
    ) -> Result<(), VkError> {
        self.device.destroy_buffer(buffer, None);
        self.device.free_memory(allocation.memory, None);
        Ok(())
    }

    pub unsafe fn create_image(
        &self,
        image_info: &vk::ImageCreateInfo,
        memory_info: &MemoryAllocationCreateInfo
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

    pub unsafe fn destroy_image(
        &self,
        image: vk::Image,
        allocation: &MemoryAllocation
    ) -> Result<(), VkError> {
        self.device.destroy_image(image, None);
        self.device.free_memory(allocation.memory, None);
        Ok(())
    }

    pub unsafe fn map_memory<T>(&self, allocation: &MemoryAllocation) -> Result<*mut T, VkError> {
        let data_ptr = self.device
            .map_memory(allocation.memory, 0, allocation.size, vk::MemoryMapFlags::empty())
            .map_err(|e| {
                VkError::OpFailed(format!("Error mapping memory: {:?}", e))
            })?;
        Ok(data_ptr as *mut T)
    }

    pub unsafe fn unmap_memory(&self, allocation: &MemoryAllocation) -> Result<(), VkError> {
        self.device.unmap_memory(allocation.memory);
        Ok(())
    }
}

pub struct MemoryAllocationCreateInfo {
    pub usage: MemoryUsage
}

pub struct MemoryAllocation {
    memory: vk::DeviceMemory,
    size: vk::DeviceSize
}

impl MemoryAllocation {
    pub fn null() -> Self {
        Self {
            memory: vk::DeviceMemory::null(),
            size: 0
        }
    }
}
