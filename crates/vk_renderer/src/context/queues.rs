
use crate::VkError;
use ash::{
    Device,
    vk
};

#[derive(Copy, Clone)]
pub struct Queue {
    pub queue_family_index: u32,
    queue: vk::Queue,
    command_buffer_pool: vk::CommandPool
}

impl Queue {

    pub unsafe fn new(device: &Device, queue_family_index: u32) -> Result<Self, VkError> {

        // Get queue
        let queue = device.get_device_queue(queue_family_index, 0);

        // One command buffer pool per queue family
        let pool_info = vk::CommandPoolCreateInfo::builder()
            .queue_family_index(queue_family_index)
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);
        let command_buffer_pool = device
            .create_command_pool(&pool_info, None)
            .map_err(|e| {
                VkError::OpFailed(format!("{:?}", e))
            })?;

        Ok(Self {
            queue_family_index,
            queue,
            command_buffer_pool
        })
    }

    pub fn get_queue(&self) -> vk::Queue {
        self.queue
    }

    pub unsafe fn allocate_command_buffer(&self, device: &Device) -> Result<vk::CommandBuffer, VkError> {
        let command_buffer_alloc_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(self.command_buffer_pool)
            .command_buffer_count(1);
        let command_buffer = device
            .allocate_command_buffers(&command_buffer_alloc_info)
            .map_err(|e| {
                VkError::OpFailed(format!("Error allocating command buffer: {:?}", e))
            })?[0];
        Ok(command_buffer)
    }

    pub unsafe fn regenerate_command_buffers(
        &self,
        device: &Device,
        buffer_count: usize
    ) -> Result<Vec<vk::CommandBuffer>, VkError> {
        device
            .reset_command_pool(
                self.command_buffer_pool,
                vk::CommandPoolResetFlags::RELEASE_RESOURCES
            )
            .map_err(|e| {
                VkError::OpFailed(format!("Error resetting command pool: {:?}", e))
            })?;
        let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(self.command_buffer_pool)
            .command_buffer_count(buffer_count as u32);
        device
            .allocate_command_buffers(&command_buffer_allocate_info)
            .map_err(|e| {
                VkError::OpFailed(format!("Error re-allocating command buffers: {:?}", e))
            })
    }

    pub unsafe fn submit_transfer_command_buffer(
        &self,
        device: &Device,
        command_buffer: &vk::CommandBuffer,
        fence: &vk::Fence
    ) -> Result<(), VkError> {
        let submit_infos = [
            vk::SubmitInfo::builder()
                .command_buffers(&[command_buffer.clone()])
                .build()
        ];
        device
            .queue_submit(self.queue, &submit_infos, fence.clone())
            .map_err(|e| {
                VkError::OpFailed(format!("Error submitting to queue: {:?}", e))
            })?;
        Ok(())
    }

    pub unsafe fn submit_graphics_command_buffer(
        &self,
        device: &Device,
        command_buffer: vk::CommandBuffer,
        sync_image_available: vk::Semaphore,
        sync_may_begin_rendering: vk::Fence,
        sync_rendering_finished: vk::Semaphore
    ) -> Result<(), VkError> {
        let semaphores_available = [sync_image_available];
        let waiting_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let semaphores_finished = [sync_rendering_finished];
        let command_buffers = [command_buffer];
        let submit_info = [vk::SubmitInfo::builder()
            .wait_semaphores(&semaphores_available)
            .wait_dst_stage_mask(&waiting_stages)
            .command_buffers(&command_buffers)
            .signal_semaphores(&semaphores_finished)
            .build()];
        device.queue_submit(
            self.queue,
            &submit_info,
            sync_may_begin_rendering
        )
            .map_err(|e| {
                VkError::OpFailed(format!("Queue submit error: {:?}", e))
            })?;
        Ok(())
    }

    pub unsafe fn free_command_buffer(&self, device: &Device, command_buffer: vk::CommandBuffer) {
        device.free_command_buffers(
            self.command_buffer_pool,
            &[command_buffer]);
    }

    pub unsafe fn destroy(&self, device: &Device) {
        device.destroy_command_pool(self.command_buffer_pool, None);
    }
}
