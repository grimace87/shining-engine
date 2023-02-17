
use crate::{StockTimer, Timer, Scene};
use vk_renderer::{VkError, VkCore, VkContext, PresentResult};
use window::{Window, PhysicalSize};
use resource::{ResourceManager, RawResourceBearer};
use std::cell::RefCell;

pub struct EngineInternals {
    timer: StockTimer,
    last_known_client_area_size: PhysicalSize<u32>,
    render_core: RefCell<VkCore>,
    render_context: RefCell<VkContext>,
    resource_manager: RefCell<ResourceManager<VkContext>>
}

impl EngineInternals {

    pub fn new<T: Sized>(window: &Window, scene: &Box<dyn RawResourceBearer<T>>) -> Result<Self, VkError> {
        // Creation of required components
        let core = unsafe { VkCore::new(&window, vec![]).unwrap() };
        let context = VkContext::new(&core, &window).unwrap();
        let mut resource_manager = ResourceManager::new();

        // Load needed resources
        let current_extent = context.get_extent()?;
        resource_manager.load_static_resources_from(&context, scene).unwrap();
        resource_manager
            .load_dynamic_resources_from(
                &context,
                scene,
                context.get_swapchain_image_count(),
                current_extent.width,
                current_extent.height)
            .unwrap();

        // Initialisation
        Ok(Self {
            timer: StockTimer::new(),
            last_known_client_area_size: PhysicalSize::default(),
            render_core: RefCell::new(core),
            render_context: RefCell::new(context),
            resource_manager: RefCell::new(resource_manager)
        })
    }

    pub fn engine_teardown(&mut self) {

        unsafe {
            self.render_context.borrow().wait_until_device_idle().unwrap();
        }

        // Free resources that the resource manager depends on
        // Note buffers and things should only be destroyed after command buffers that reference
        // them have been destroyed or reset
        self.render_context.borrow_mut().release_command_buffers().unwrap();

        // Free resources
        self.resource_manager.borrow_mut()
            .free_resources(&mut self.render_context.borrow_mut()).unwrap();

        // Destroy renderer
        self.render_context.borrow_mut().teardown();
        self.render_core.borrow_mut().teardown();
    }

    pub fn record_graphics_commands<T: Sized>(
        &self,
        scene: &Box<dyn Scene<T>>
    ) -> Result<(), VkError> {
        let context = self.render_context.borrow();
        let resource_manager = self.resource_manager.borrow();
        for image_index in 0..context.get_swapchain_image_count() {
            let command_buffer = context.get_graphics_command_buffer(image_index);
            unsafe {
                scene.record_commands(
                    &context.device,
                    command_buffer,
                    context.get_extent()?,
                    &resource_manager,
                    image_index)?;
            }
        }
        Ok(())
    }

    pub fn pull_time_step_millis(&mut self) -> u64 {
        self.timer.pull_time_step_millis()
    }

    pub fn get_last_known_size(&self) -> PhysicalSize<u32> {
        self.last_known_client_area_size
    }

    pub fn recreate_surface<T: Sized>(
        &mut self,
        window: &Window,
        new_client_area_size: PhysicalSize<u32>,
        scene: &Box<dyn Scene<T>>
    ) -> Result<(), VkError> {
        // Wait for the device to be idle
        unsafe {
            self.render_context.borrow().wait_until_device_idle()?;
        }

        // Get needed things
        let core = self.render_core.borrow();
        let resource_bearer = scene.get_resource_bearer();

        // Recreate everything
        unsafe {
            let mut context = self.render_context.borrow_mut();
            let mut resource_manager = self.resource_manager.borrow_mut();
            let current_extent = context.get_extent()?;
            context.recreate_surface(&core, window)?;
            context.regenerate_graphics_command_buffers()?;
            resource_manager.release_swapchain_dynamic_resources(&mut context)?;
            resource_manager
                .load_dynamic_resources_from(
                    &context,
                    &resource_bearer,
                    context.get_swapchain_image_count(),
                    current_extent.width,
                    current_extent.height)?;
        }
        self.record_graphics_commands(scene)?;
        self.last_known_client_area_size = new_client_area_size;
        Ok(())
    }

    pub fn render_frame<T: Sized>(&mut self, scene: &Box<dyn Scene<T>>) -> Result<PresentResult, VkError> {
        let mut context = self.render_context.borrow_mut();
        let resource_manager = self.resource_manager.borrow();
        unsafe {
            let (image_index, up_to_date) = context.acquire_next_image()?;
            if !up_to_date {
                return Ok(PresentResult::SwapchainOutOfDate);
            }

            scene.prepare_frame_render(&context, image_index, &resource_manager)?;
            context.submit_and_present()
        }
    }
}
