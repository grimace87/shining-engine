
use crate::mem::{
    MemoryAllocator, ManagesMemoryTransfers, MemoryAllocation
};
use crate::{VkError, Queue};

use ash::vk;

impl ManagesMemoryTransfers for MemoryAllocator {

    unsafe fn transfer_data_to_new_texture(
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
}
