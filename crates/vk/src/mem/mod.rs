
use crate::{VkError, Queue};

use ash::{Device, Instance, vk};

const BULK_MEMORY_USABLE_MINIMUM: vk::DeviceSize = 536_870_912;
const INITIAL_STAGING_BUFFER_SIZE: vk::DeviceSize = 134_217_728;

pub enum MemoryUsage {
    CpuToGpu,
    GpuOnly
}

struct MemoryAllocationParameters {
    memory_type_bulk_performance: u32,
    memory_type_uniform_buffer: u32,
    memory_type_staging_buffer: Option<u32>,
    prefer_image_tiling: bool
}

struct StagingBufferParameters {
    memory_type: u32,
    buffer: vk::Buffer,
    allocation: MemoryAllocation
}

pub struct MemoryAllocatorCreateInfo {
    pub physical_device: vk::PhysicalDevice,
    pub device: Device,
    pub instance: Instance,
    pub transfer_command_buffer: vk::CommandBuffer
}

pub struct MemoryAllocator {
    physical_device: vk::PhysicalDevice,
    device: Device,
    allocation_parameters: MemoryAllocationParameters,
    transfer_command_buffer: vk::CommandBuffer,
    staging_buffer_parameters: Option<StagingBufferParameters>
}

/// Memory allocator for buffers and images.
/// Logic for compatibility and optimal performance on a range of devices is based on this guide:
/// https://asawicki.info/news_1740_vulkan_memory_types_on_pc_and_how_to_use_them
///
/// Intel-like devices (one heap, all host-visible and device-local) don't need a staging buffer
/// but should use images with VK_IMAGE_TILING_OPTIMAL.
///
/// NVIDIA-like devices (two heaps, one host-visible and one device-local with no overlap) need a
/// staging buffer.
///
/// AMD-like devices (like NVIDIA, plus an extra heap that is both host-visible and device-local
/// but very small) can put things like uniform buffers in that extra heap (ideally check budget).
///
/// SAM (or ReBAR) devices will not be specifically accounted for here, but should work fine with
/// the other logic.
///
/// APU-like devices (two heaps similar to NVIDIA but the device-local one is quite small) should
/// limit their use of device-local memory, hoping that other memory is at least nearly as fast.
impl MemoryAllocator {

    pub unsafe fn new(allocator_info: MemoryAllocatorCreateInfo) -> Result<Self, VkError> {

        // Gather some info on the device's memory; will decide how to allocate memory later
        let memory_properties = allocator_info.instance
            .get_physical_device_memory_properties(allocator_info.physical_device);
        let allocation_parameters = Self::select_memory_types(memory_properties)?;
        let staging_buffer_parameters = match allocation_parameters.memory_type_staging_buffer {
            Some(memory_type) => Some(
                Self::create_staging_buffer_parameters(&allocator_info.device, memory_type)?),
            None => None
        };

        Ok(Self {
            physical_device: allocator_info.physical_device,
            device: allocator_info.device,
            allocation_parameters,
            transfer_command_buffer: allocator_info.transfer_command_buffer,
            staging_buffer_parameters
        })
    }

    pub unsafe fn destroy(&mut self, transfer_queue: &Queue) {
        if let Some(staging_buffer_parameters) = &self.staging_buffer_parameters {
            self.device.destroy_buffer(staging_buffer_parameters.buffer, None);
            self.device.free_memory(staging_buffer_parameters.allocation.memory, None);
        }
        transfer_queue.free_command_buffer(&self.device, self.transfer_command_buffer);
    }

    /// Return appropriate memory types for various purposes, or an error
    /// - Bulk performance memory (long-lived, static buffers and images accessed only by GPU)
    /// - Uniform buffer memory (buffers often written to by CPU and accessed by GPU)
    /// - Staging buffer memory (buffers written to by CPU and only immediately used in a transfer)
    unsafe fn select_memory_types(
        memory_properties: vk::PhysicalDeviceMemoryProperties
    ) -> Result<MemoryAllocationParameters, VkError> {
        let mut has_device_local_only = false;
        let mut device_local_only_index: u32 = 0;
        let mut device_local_only_size: vk::DeviceSize = 0;
        let mut has_host_accessible_only = false;
        let mut host_accessible_only_index: u32 = 0;
        let mut host_accessible_only_size: vk::DeviceSize = 0;
        let mut has_flexible_memory = false;
        let mut flexible_memory_index: u32 = 0;
        let mut flexible_memory_size: vk::DeviceSize = 0;
        for memory_type in 0..memory_properties.memory_type_count {

            // Collect info on this memory type
            let heap_index = memory_properties.memory_types[memory_type as usize].heap_index;
            let heap_size = memory_properties.memory_heaps[heap_index as usize].size;
            let flags = memory_properties.memory_types[memory_type as usize].property_flags;
            let is_local = (flags & vk::MemoryPropertyFlags::DEVICE_LOCAL) != vk::MemoryPropertyFlags::empty();
            let is_accessible = (flags & vk::MemoryPropertyFlags::HOST_VISIBLE) != vk::MemoryPropertyFlags::empty() &&
                (flags & vk::MemoryPropertyFlags::HOST_COHERENT) != vk::MemoryPropertyFlags::empty();

            // Logic for selecting memory types to use
            if is_local && is_accessible {
                if heap_size > flexible_memory_size {
                    has_flexible_memory = true;
                    flexible_memory_index = memory_type;
                    flexible_memory_size = heap_size;
                }
            } else if is_local {
                if heap_size > device_local_only_size {
                    has_device_local_only = true;
                    device_local_only_index = memory_type;
                    device_local_only_size = heap_size;
                }
            } else if is_accessible {
                if heap_size > host_accessible_only_size {
                    has_host_accessible_only = true;
                    host_accessible_only_index = memory_type;
                    host_accessible_only_size = heap_size;
                }
            }
        }

        // Decide which memory types to use for different things
        let mut chosen_type_bulk_performance: Option<u32> = None;
        let mut chosen_type_uniform_buffer: Option<u32> = None;
        let mut chosen_type_staging_buffer: Option<u32> = None;
        let mut prefer_image_tiling = false;

        // Scenarios where there's nothing specialised for host accessibility (all device-local)
        if !has_host_accessible_only {
            if !has_flexible_memory {
                return if has_device_local_only {
                    Err(VkError::Compatibility("No host-accessible memory found".to_owned()))
                } else {
                    Err(VkError::Compatibility("No memory types were found".to_owned()))
                };
            }
            if has_device_local_only {
                // All memory device-local, some is also host-accessible (very unusual case?)
                chosen_type_bulk_performance = Some(device_local_only_index);
                chosen_type_uniform_buffer = Some(flexible_memory_index);
                chosen_type_staging_buffer = Some(flexible_memory_index);
            } else {
                // All memory both device-local and host-accessible (Intel-like)
                chosen_type_bulk_performance = Some(flexible_memory_index);
                chosen_type_uniform_buffer = Some(flexible_memory_index);
                chosen_type_staging_buffer = None;
                prefer_image_tiling = true;
            }
        }

        // Scenarios where some memory is host-accessible but not device-local
        else {
            if !has_device_local_only && !has_flexible_memory {
                return Err(VkError::Compatibility("No device-local memory found".to_owned()));
            }
            if !has_device_local_only {
                // All memory host-accessible, some is also device-local (very unusual case?)
                chosen_type_bulk_performance = Some(flexible_memory_index);
                chosen_type_uniform_buffer = Some(flexible_memory_index);
                chosen_type_staging_buffer = Some(host_accessible_only_index);
            } else if !has_flexible_memory {
                // Memory is either host-accessible or device-local, never both (NVIDIA-like)
                chosen_type_bulk_performance = Some(device_local_only_index);
                chosen_type_uniform_buffer = Some(host_accessible_only_index);
                chosen_type_staging_buffer = Some(host_accessible_only_index);
            } else {
                // Some device-local only, some host-accessible only, some that's everything
                if device_local_only_size >= BULK_MEMORY_USABLE_MINIMUM {
                    chosen_type_bulk_performance = Some(device_local_only_index);
                    chosen_type_uniform_buffer = Some(flexible_memory_index);
                    if flexible_memory_size >= BULK_MEMORY_USABLE_MINIMUM {
                        chosen_type_staging_buffer = Some(flexible_memory_index);
                    } else {
                        // AMD-like
                        chosen_type_staging_buffer = Some(host_accessible_only_index);
                    }
                } else {
                    chosen_type_uniform_buffer = Some(flexible_memory_index);
                    chosen_type_staging_buffer = None;
                    if host_accessible_only_size >= BULK_MEMORY_USABLE_MINIMUM {
                        // APU-like
                        chosen_type_bulk_performance = Some(host_accessible_only_index);
                    } else {
                        chosen_type_bulk_performance = Some(flexible_memory_index);
                    }
                }
            }
        }

        let Some(performance_type) = chosen_type_bulk_performance else {
            return Err(VkError::Compatibility("Logic error selecting memory".to_owned()));
        };
        let Some(uniform_type) = chosen_type_uniform_buffer else {
            return Err(VkError::Compatibility("Logic error selecting memory".to_owned()));
        };
        Ok(MemoryAllocationParameters {
            memory_type_bulk_performance: performance_type,
            memory_type_uniform_buffer: uniform_type,
            memory_type_staging_buffer: chosen_type_staging_buffer,
            prefer_image_tiling
        })
    }

    unsafe fn create_staging_buffer_parameters(
        device: &Device,
        memory_type: u32
    ) -> Result<StagingBufferParameters, VkError> {

        let buffer_create_info = vk::BufferCreateInfo::builder()
            .size(INITIAL_STAGING_BUFFER_SIZE)
            .usage(vk::BufferUsageFlags::TRANSFER_SRC)
            .build();
        let buffer = device.create_buffer(&buffer_create_info, None)
            .map_err(|e| VkError::OpFailed(
                format!("Failed to create staging buffer: {:?}", e)
            ))?;

        let requirements = device.get_buffer_memory_requirements(buffer);
        let allocate_info = vk::MemoryAllocateInfo::builder()
            .allocation_size(requirements.size)
            .memory_type_index(memory_type)
            .build();
        let memory = device.allocate_memory(&allocate_info, None)
            .map_err(|e| {
                VkError::OpFailed(format!("Error allocating staging buffer memory: {:?}", e))
            })?;

        device.bind_buffer_memory(buffer, memory, 0)
            .map_err(|e| VkError::OpFailed(
                format!("Error binding staging buffer memory: {:?}", e)
            ))?;

        Ok(StagingBufferParameters {
            memory_type,
            buffer,
            allocation: MemoryAllocation {
                size: requirements.size,
                memory
            }
        })
    }

    pub unsafe fn transfer_data_to_new_texture(
        &self,
        transfer_queue: &Queue,
        width: u32,
        height: u32,
        image_dst: &vk::Image,
        allocation: &MemoryAllocation,
        layer_data: &[Vec<u8>]
    ) -> Result<(), VkError> {

        let layer_count = layer_data.len();
        let layer_size_bytes = layer_data[0].len();

        // Staging buffer
        let expected_data_size: usize = layer_count * 4 * width as usize * height as usize;
        if expected_data_size != layer_count * layer_size_bytes {
            panic!("Image data does not match expected size");
        }

        if self.staging_buffer_parameters.is_some() {
            self.transfer_data_to_new_texture_with_staging_buffer(
                transfer_queue, width, height, image_dst, layer_data)
        } else {
            self.transfer_data_to_new_texture_without_staging_buffer(
                transfer_queue, image_dst, allocation, layer_data)
        }
    }

    unsafe fn transfer_data_to_new_texture_without_staging_buffer(
        &self,
        transfer_queue: &Queue,
        image_dst: &vk::Image,
        allocation: &MemoryAllocation,
        layer_data: &[Vec<u8>]
    ) -> Result<(), VkError> {

        // Copy data into image memory
        let layer_count = layer_data.len();
        let layer_size_bytes = layer_data[0].len();
        for (layer_no, data) in layer_data.iter().enumerate() {
            let src_ptr = data.as_ptr() as *const u8;
            let mut dst_ptr = self.map_memory::<u8>(allocation)?;
            let dst_offset_elements = (layer_no * layer_size_bytes) as isize;
            dst_ptr = dst_ptr.offset(dst_offset_elements);
            dst_ptr.copy_from_nonoverlapping(src_ptr, layer_size_bytes);
            self.unmap_memory(&allocation).unwrap();
        }

        // Allocate a single-use command buffer and begin recording
        let command_begin_info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
        self.device.begin_command_buffer(self.transfer_command_buffer, &command_begin_info)
            .map_err(|e| {
                VkError::OpFailed(format!("Error starting copy command buffer: {:?}", e))
            })?;

        // Memory dependency - move to final image layout
        let barrier = vk::ImageMemoryBarrier::builder()
            .image(*image_dst)
            .src_access_mask(vk::AccessFlags::empty())
            .dst_access_mask(vk::AccessFlags::MEMORY_READ)
            .old_layout(vk::ImageLayout::PREINITIALIZED)
            .new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: layer_count as u32
            })
            .build();
        self.device.cmd_pipeline_barrier(
            self.transfer_command_buffer,
            vk::PipelineStageFlags::TOP_OF_PIPE,
            vk::PipelineStageFlags::TRANSFER,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            &[barrier]
        );

        // Finish recording commands, create a fence, run the command, wait for fence, clean up
        self.device.end_command_buffer(self.transfer_command_buffer)
            .map_err(|e| {
                VkError::OpFailed(format!("Error ending command buffer: {:?}", e))
            })?;
        let fence = self.device
            .create_fence(&vk::FenceCreateInfo::default(), None)
            .map_err(|e| {
                VkError::OpFailed(format!("Error creating fence: {:?}", e))
            })?;
        transfer_queue.submit_command_buffer(
            &self.device,
            &self.transfer_command_buffer,
            &fence)?;
        self.device
            .wait_for_fences(&[fence], true, u64::MAX)
            .map_err(|e| {
                VkError::OpFailed(format!("Error waiting for fence: {:?}", e))
            })?;
        self.device
            .destroy_fence(fence, None);

        Ok(())
    }

    unsafe fn transfer_data_to_new_texture_with_staging_buffer(
        &self,
        transfer_queue: &Queue,
        width: u32,
        height: u32,
        image_dst: &vk::Image,
        layer_data: &[Vec<u8>]
    ) -> Result<(), VkError> {

        let Some(staging_parameters) = &self.staging_buffer_parameters else {
            return Err(VkError::OpFailed(
                "Internal error: transferring from staging without a buffer".to_owned()
            ));
        };

        // Copy data into staging buffer
        let layer_size_bytes = layer_data[0].len();
        let layer_count = layer_data.len();
        for (layer_no, data) in layer_data.iter().enumerate() {
            let src_ptr = data.as_ptr() as *const u8;
            let mut dst_ptr = self.map_memory::<u8>(&staging_parameters.allocation)?;
            let dst_offset_elements = (layer_no * layer_size_bytes) as isize;
            dst_ptr = dst_ptr.offset(dst_offset_elements);
            dst_ptr.copy_from_nonoverlapping(src_ptr, layer_size_bytes);
            self.unmap_memory(&staging_parameters.allocation).unwrap();
        }

        // Allocate a single-use command buffer and begin recording
        let command_begin_info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
        self.device.begin_command_buffer(self.transfer_command_buffer, &command_begin_info)
            .map_err(|e| {
                VkError::OpFailed(format!("Error starting copy command buffer: {:?}", e))
            })?;

        // Initial memory dependency
        let barrier = vk::ImageMemoryBarrier::builder()
            .image(*image_dst)
            .src_access_mask(vk::AccessFlags::empty())
            .dst_access_mask(vk::AccessFlags::TRANSFER_WRITE)
            .old_layout(vk::ImageLayout::UNDEFINED)
            .new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
            .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: layer_count as u32
            })
            .build();
        self.device.cmd_pipeline_barrier(
            self.transfer_command_buffer,
            vk::PipelineStageFlags::TOP_OF_PIPE,
            vk::PipelineStageFlags::TRANSFER,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            &[barrier]
        );

        // Copy command
        let image_subresource = vk::ImageSubresourceLayers {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            mip_level: 0,
            base_array_layer: 0,
            layer_count: layer_count as u32
        };
        let region = vk::BufferImageCopy {
            buffer_offset: 0,
            buffer_row_length: 0,
            buffer_image_height: 0,
            image_offset: vk::Offset3D { x: 0, y: 0, z: 0 },
            image_extent: vk::Extent3D { width, height, depth: 1 },
            image_subresource
        };
        self.device.cmd_copy_buffer_to_image(
            self.transfer_command_buffer,
            staging_parameters.buffer,
            *image_dst,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            &[region]
        );

        // Final memory dependency
        let barrier = vk::ImageMemoryBarrier::builder()
            .image(*image_dst)
            .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
            .dst_access_mask(vk::AccessFlags::MEMORY_READ)
            .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
            .new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: layer_count as u32
            })
            .build();
        self.device.cmd_pipeline_barrier(
            self.transfer_command_buffer,
            vk::PipelineStageFlags::TRANSFER,
            vk::PipelineStageFlags::BOTTOM_OF_PIPE,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            &[barrier]
        );

        // Finish recording commands, create a fence, run the command, wait for fence, clean up
        self.device.end_command_buffer(self.transfer_command_buffer)
            .map_err(|e| {
                VkError::OpFailed(format!("Error ending command buffer: {:?}", e))
            })?;
        let fence = self.device
            .create_fence(&vk::FenceCreateInfo::default(), None)
            .map_err(|e| {
                VkError::OpFailed(format!("Error creating fence: {:?}", e))
            })?;
        transfer_queue.submit_command_buffer(
            &self.device,
            &self.transfer_command_buffer,
            &fence)?;
        self.device
            .wait_for_fences(&[fence], true, u64::MAX)
            .map_err(|e| {
                VkError::OpFailed(format!("Error waiting for fence: {:?}", e))
            })?;
        self.device
            .destroy_fence(fence, None);

        Ok(())
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
