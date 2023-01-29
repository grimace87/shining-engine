mod device;
mod present;
mod queues;
mod swapchain;

use crate::{
    VkError,
    VkCore,
    ImageWrapper,
    mem::{MemoryAllocator, MemoryAllocatorCreateInfo}
};
use ash::{
    Device,
    extensions::khr::{
        Surface,
        Swapchain
    },
    vk
};
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};

pub use present::PresentResult;
pub use queues::Queue;
pub use swapchain::SwapchainWrapper;

/// Wrap logical device along with Vulkan components that can exist for the life of a window
pub struct VkContext {
    pub device: Device,
    borrowed_physical_device_handle: vk::PhysicalDevice,
    pub graphics_queue: Queue,
    pub transfer_queue: Queue,
    graphics_command_buffers: Vec<vk::CommandBuffer>,
    mem_allocator: MemoryAllocator,
    sync_image_available: Vec<vk::Semaphore>,
    sync_may_begin_rendering: Vec<vk::Fence>,
    sync_rendering_finished: Vec<vk::Semaphore>,
    current_image_acquired: usize,
    surface_fn: Surface,
    surface: vk::SurfaceKHR,
    swapchain_fn: Swapchain,
    swapchain: SwapchainWrapper,
}

impl VkContext {

    pub fn new<T>(core: &VkCore, window: &T) -> Result<Self, VkError>
        where T: HasRawDisplayHandle + HasRawWindowHandle
    {
        Ok(unsafe {
            let mut context = Self::new_with_surface_without_swapchain(core, window)?;
            context.create_swapchain(core)?;
            context.regenerate_graphics_command_buffers()?;
            context
        })
    }

    pub fn teardown(&mut self) {
        unsafe {
            self.destroy_swapchain_resources();
            self.surface_fn.destroy_surface(self.surface, None);
            self.mem_allocator.destroy(&self.transfer_queue);
            self.transfer_queue.destroy(&self.device);
            self.graphics_queue.destroy(&self.device);
            self.device.destroy_device(None);
        }
    }

    /// Create a new instance, but not yet creating the swapchain. For internal use.
    unsafe fn new_with_surface_without_swapchain<T>(
        core: &VkCore,
        window: &T
    ) -> Result<VkContext, VkError>
        where T: HasRawDisplayHandle + HasRawWindowHandle
    {

        // Create surface and surface loader
        let surface_fn = Surface::new(&core.function_loader, &core.instance);
        let surface = ash_window::create_surface(
            &core.function_loader,
            &core.instance,
            window.raw_display_handle(),
            window.raw_window_handle(),
            None)
            .map_err(|e| VkError::OpFailed(format!("Error creating surface: {}", e)))?;

        // Create device
        let device = device::make_device_resources(core)?;

        // Make queues
        let graphics_queue = Queue::new(&device, core.graphics_queue_family_index)?;
        let transfer_queue = Queue::new(&device, core.transfer_queue_family_index)?;

        // Allocate a command buffer for the transfer queue
        let transfer_command_buffer = transfer_queue
            .allocate_command_buffer(&device)?;

        // Create a memory allocator
        let allocator_info = MemoryAllocatorCreateInfo {
            physical_device: core.physical_device,
            device: device.clone(),
            instance: core.instance.clone(),
            transfer_command_buffer
        };
        let mem_allocator = MemoryAllocator::new(allocator_info)?;

        let swapchain_fn = Swapchain::new(&core.instance, &device);

        Ok(
            Self {
                device,
                borrowed_physical_device_handle: core.physical_device,
                graphics_queue,
                transfer_queue,
                graphics_command_buffers: vec![],
                mem_allocator,
                sync_image_available: vec![],
                sync_may_begin_rendering: vec![],
                sync_rendering_finished: vec![],
                current_image_acquired: 0,
                surface_fn,
                surface,
                swapchain_fn,
                swapchain: SwapchainWrapper::default()
            }
        )
    }

    /// Get the dimensions of the current surface
    pub fn get_extent(&self) -> Result<vk::Extent2D, VkError> {
        let surface_capabilities = unsafe {
            self.surface_fn.get_physical_device_surface_capabilities(
                self.borrowed_physical_device_handle,
                self.surface
            )
                .map_err(|e| {
                    VkError::OpFailed(format!("{:?}", e))
                })?
        };
        Ok(surface_capabilities.current_extent)
    }

    /// Getter for swapchain image count
    pub fn get_swapchain_image_count(&self) -> usize {
        self.swapchain.get_image_count()
    }

    /// Getter for a swapchain image view
    pub fn get_swapchain_image_view(&self, image_index: usize) -> Result<vk::ImageView, VkError> {
        self.swapchain.get_image_view(image_index)
    }

    /// Getter for the depth image
    pub fn get_depth_image(&self) -> Option<&ImageWrapper> {
        self.swapchain.get_depth_image()
    }

    /// Query the surface format used by the current swapchain
    pub unsafe fn get_surface_format(&self) -> vk::SurfaceFormatKHR {
        self.swapchain.get_surface_format()
    }

    /// Create the swapchain; any previously-created swapchain should be destroyed first
    unsafe fn create_swapchain(&mut self, core: &VkCore) -> Result<(), VkError> {

        let extent = self.get_extent()?;
        self.swapchain = SwapchainWrapper::new(core, &self, &self.surface_fn, self.surface, extent)?;
        self.current_image_acquired = self.swapchain.get_image_count() - 1;

        // Synchronisation objects
        self.sync_image_available.clear();
        self.sync_may_begin_rendering.clear();
        self.sync_rendering_finished.clear();
        let swapchain_size = self.swapchain.get_image_count();
        let semaphore_create_info = vk::SemaphoreCreateInfo::builder();
        let fence_create_info = vk::FenceCreateInfo::builder()
            .flags(vk::FenceCreateFlags::SIGNALED);
        for _ in 0..swapchain_size {
            let semaphore_available = self.device
                .create_semaphore(&semaphore_create_info, None)
                .map_err(|e| {
                    VkError::OpFailed(format!("{:?}", e))
                })?;
            let fence_begin_rendering = self.device
                .create_fence(&fence_create_info, None)
                .map_err(|e| {
                    VkError::OpFailed(format!("{:?}", e))
                })?;
            let semaphore_finished = self.device
                .create_semaphore(&semaphore_create_info, None)
                .map_err(|e| {
                    VkError::OpFailed(format!("{:?}", e))
                })?;
            self.sync_image_available.push(semaphore_available);
            self.sync_may_begin_rendering.push(fence_begin_rendering);
            self.sync_rendering_finished.push(semaphore_finished);
        }

        Ok(())
    }

    /// Destroy resources associated with the swapchain
    unsafe fn destroy_swapchain_resources(&mut self) {
        for semaphore in self.sync_rendering_finished.iter() {
            self.device.destroy_semaphore(*semaphore, None);
        }
        for fence in self.sync_may_begin_rendering.iter() {
            self.device.destroy_fence(*fence, None);
        }
        for semaphore in self.sync_image_available.iter() {
            self.device.destroy_semaphore(*semaphore, None);
        }
        self.swapchain.destroy(&self, &self.swapchain_fn);
    }

    /// Getter for the memory allocator
    pub fn get_mem_allocator(&self) -> (&MemoryAllocator, &Queue) {
        (&self.mem_allocator, &self.transfer_queue)
    }

    pub unsafe fn wait_until_device_idle(&self) -> Result<(), VkError> {
        self.device.device_wait_idle()
            .map_err(|e| {
                VkError::OpFailed(format!("Failed waiting for device: {:?}", e))
            })?;
        Ok(())
    }

    /// Frees the set of graphics command buffers and generates a new set. So long as we call
    /// vkResetCommandPool before creating new command buffers, we don't need to free each one
    /// of the old ones individually.
    pub unsafe fn regenerate_graphics_command_buffers(
        &mut self
    ) -> Result<(), VkError> {
        self.graphics_command_buffers.clear();
        let graphics_command_buffers = self.graphics_queue.regenerate_command_buffers(
            &self.device,
            self.swapchain.get_image_count())?;
        self.graphics_command_buffers.extend(graphics_command_buffers);
        Ok(())
    }

    pub fn get_graphics_command_buffer(&self, swapchain_image_index: usize) -> vk::CommandBuffer {
        self.graphics_command_buffers[swapchain_image_index]
    }

    pub unsafe fn recreate_surface<T>(
        &mut self,
        core: &VkCore,
        window: &T
    ) -> Result<(), VkError>
        where T: HasRawDisplayHandle + HasRawWindowHandle
    {
        self.destroy_swapchain_resources();
        self.surface_fn.destroy_surface(self.surface, None);
        self.surface = ash_window::create_surface(
            &core.function_loader,
            &core.instance,
            window.raw_display_handle(),
            window.raw_window_handle(),
            None)
            .map_err(|e| VkError::OpFailed(format!("Error creating surface: {}", e)))?;
        self.create_swapchain(core)?;
        Ok(())
    }

    // Increment current image number to focus on the next image in the chain, to wait for its
    // synchronisation objects and so on.
    //
    // Acquires an image while signalling a semaphore, then waits on a fence to know that the
    // image is available to draw on.
    pub unsafe fn acquire_next_image(&mut self) -> Result<(usize, bool), VkError> {
        let swapchain_size = self.get_swapchain_image_count();
        let sync_objects_index = (self.current_image_acquired + 1) % swapchain_size;
        let result = self.swapchain_fn.acquire_next_image(
            self.swapchain.get_swapchain(),
            u64::MAX,
            self.sync_image_available[sync_objects_index],
            vk::Fence::null());
        let (image_index, _) = match result {
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => return Ok((0, false)),
            Err(e) => return Err(VkError::OpFailed(
                format!("Image acquire failure: {:?}", e))),
            Ok(t) => t
        };
        self.current_image_acquired = image_index as usize;
        assert_eq!(sync_objects_index, image_index as usize);

        self.device.wait_for_fences(
            &[self.sync_may_begin_rendering[self.current_image_acquired]],
            true,
            u64::MAX)
            .map_err(|e| {
                VkError::OpFailed(format!("Waiting on fence error: {:?}", e))
            })?;
        self.device.reset_fences(&[self.sync_may_begin_rendering[self.current_image_acquired]])
            .map_err(|e| {
                VkError::OpFailed(format!("Resetting fence error: {:?}", e))
            })?;

        Ok((self.current_image_acquired, true))
    }

    pub unsafe fn submit_and_present(&self) -> Result<PresentResult, VkError> {

        // Submit graphics work
        let command_buffer = self.graphics_command_buffers[self.current_image_acquired];
        let sync_image_available = self.sync_image_available[self.current_image_acquired];
        let sync_may_begin_rendering = self.sync_may_begin_rendering[self.current_image_acquired];
        let sync_rendering_finished = self.sync_rendering_finished[self.current_image_acquired];
        self.graphics_queue.submit_graphics_command_buffer(
            &self.device,
            command_buffer,
            sync_image_available,
            sync_may_begin_rendering,
            sync_rendering_finished)?;

        // Present image
        let semaphores_finished = [self.sync_rendering_finished[self.current_image_acquired]];
        let swapchains = [self.swapchain.get_swapchain()];
        let indices = [self.current_image_acquired as u32];
        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(&semaphores_finished)
            .swapchains(&swapchains)
            .image_indices(&indices);
        let present_result = self.swapchain_fn
            .queue_present(self.graphics_queue.get_queue(), &present_info);
        return match present_result {
            Ok(_) => Ok(PresentResult::Ok),
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                Ok(PresentResult::SwapchainOutOfDate)
            },
            Err(e) => Err(VkError::OpFailed(format!("Present error: {:?}", e)))
        };
    }
}
