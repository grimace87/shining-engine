mod device;
mod swapchain;

use crate::{VkCore, VkError};
use ash::{
    Device,
    extensions::khr::{
        Surface,
        Swapchain
    },
    version::DeviceV1_0,
    vk
};
use raw_window_handle::HasRawWindowHandle;
use vk_mem::AllocatorCreateFlags;

/// Wrap logical device along with Vulkan components that can exist for the life of a window
pub struct VkContext {
    pub device: Device,
    pub graphics_queue: vk::Queue,
    pub transfer_queue: vk::Queue,
    mem_allocator: vk_mem::Allocator,
    sync_image_available: Vec<vk::Semaphore>,
    sync_may_begin_rendering: Vec<vk::Fence>,
    sync_rendering_finished: Vec<vk::Semaphore>,
    current_image_acquired: usize,
    surface_fn: Surface,
    surface: vk::SurfaceKHR,
    swapchain_fn: Swapchain,
    swapchain: vk::SwapchainKHR,
    pub image_views: Vec<vk::ImageView>
}

impl VkContext {

    pub fn new(core: &VkCore, window_owner: &dyn HasRawWindowHandle) -> Result<Self, VkError> {
        Ok(unsafe {
            let mut context = Self::new_with_surface_without_swapchain(core, window_owner)?;
            context.create_swapchain(core)?;
            context
        })
    }

    /// Create a new instance, but not yet creating the swapchain. For internal use.
    unsafe fn new_with_surface_without_swapchain(
        core: &VkCore,
        window_owner: &dyn HasRawWindowHandle
    ) -> Result<VkContext, VkError> {

        // Create surface and surface loader
        let surface_fn = Surface::new(&core.function_loader, &core.instance);
        let surface = ash_window::create_surface(
            &core.function_loader,
            &core.instance,
            window_owner,
            None)
            .map_err(|e| VkError::OpFailed(format!("Error creating surface: {}", e)))?;

        // Create device and queues
        let (device, graphics_queue, transfer_queue) =
            device::make_device_resources(core)?;

        // Create a memory allocator
        let allocator_info = vk_mem::AllocatorCreateInfo {
            physical_device: core.physical_device,
            device: device.clone(),
            instance: core.instance.clone(),
            flags: AllocatorCreateFlags::NONE,
            preferred_large_heap_block_size: 0,
            frame_in_use_count: 0,
            heap_size_limits: None
        };
        let mem_allocator = vk_mem::Allocator::new(&allocator_info)
            .map_err(|e| {
                VkError::OpFailed(format!("{:?}", e))
            })?;

        let swapchain_fn = Swapchain::new(&core.instance, &device);

        Ok(
            Self {
                device,
                graphics_queue,
                transfer_queue,
                mem_allocator,
                sync_image_available: vec![],
                sync_may_begin_rendering: vec![],
                sync_rendering_finished: vec![],
                current_image_acquired: 0,
                surface_fn,
                surface,
                swapchain_fn,
                swapchain: vk::SwapchainKHR::null(),
                image_views: vec![]
            }
        )
    }

    /// Create the swapchain; any previously-created swapchain should be destroyed first
    unsafe fn create_swapchain(&mut self, core: &VkCore) -> Result<(), VkError> {

        self.swapchain = swapchain::create_swapchain(
            core,
            &self.surface_fn,
            self.surface,
            &self.swapchain_fn,
            vk::SwapchainKHR::null())?;
        let mut swapchain_image_views =
            swapchain::create_swapchain_image_views(
                &self.device,
                &self.swapchain_fn,
                self.swapchain)?;
        self.image_views.clear();
        self.image_views.append(&mut swapchain_image_views);
        self.current_image_acquired = self.image_views.len() - 1;

        // Synchronisation objects
        self.sync_image_available.clear();
        self.sync_may_begin_rendering.clear();
        self.sync_rendering_finished.clear();
        let swapchain_size = self.image_views.len();
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
        for image_view in self.image_views.iter_mut() {
            self.device.destroy_image_view(*image_view, None);
        }
        self.swapchain_fn.destroy_swapchain(self.swapchain, None);
    }

    /// Getter for the memory allocator
    pub fn get_mem_allocator(&self) -> &vk_mem::Allocator {
        &self.mem_allocator
    }
}

impl Drop for VkContext {

    fn drop(&mut self) {
        unsafe {
            self.destroy_swapchain_resources();
            self.surface_fn.destroy_surface(self.surface, None);
            self.mem_allocator.destroy();
            self.device.destroy_device(None);
        }
    }
}
