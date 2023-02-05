
use crate::{SceneFactory, Renderable, StockTimer, Timer};
use vk_renderer::{VkError, VkCore, VkContext, PresentResult};
use window::{Window, PhysicalSize, event::{RenderEventHandler, WindowEventHandler}};
use resource::{ResourceManager, RawResourceBearer};
use std::cell::RefCell;
use std::fmt::Debug;

pub struct EngineInternals {
    timer: StockTimer,
    last_known_client_area_size: PhysicalSize<u32>,
    render_core: RefCell<VkCore>,
    render_context: RefCell<VkContext>,
    resource_manager: RefCell<ResourceManager<VkContext>>
}

impl EngineInternals {

    pub fn new<M, A>(window: &Window, app: &A) -> Result<Self, VkError> where
        M: 'static + Send + Debug,
        A: 'static + WindowEventHandler<M> + RenderEventHandler + RawResourceBearer + SceneFactory
    {
        // Creation of required components
        let core = unsafe { VkCore::new(&window, vec![]).unwrap() };
        let context = VkContext::new(&core, &window).unwrap();
        let mut resource_manager = ResourceManager::new();

        // Load needed resources
        let current_extent = context.get_extent()?;
        resource_manager.load_static_resources_from(&context, app).unwrap();
        resource_manager
            .load_dynamic_resources_from(
                &context,
                app,
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

    pub fn record_graphics_commands(
        &self,
        renderable: &Box<dyn Renderable>
    ) -> Result<(), VkError> {
        let context = self.render_context.borrow();
        let resource_manager = self.resource_manager.borrow();
        for image_index in 0..context.get_swapchain_image_count() {
            let command_buffer = context.get_graphics_command_buffer(image_index);
            unsafe {
                renderable.record_commands(
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

    pub fn recreate_surface<M, A>(
        &mut self,
        window: &Window,
        new_client_area_size: PhysicalSize<u32>,
        app: &A
    ) -> Result<(), VkError> where
        M: 'static + Send + Debug,
        A: 'static + WindowEventHandler<M> + RenderEventHandler + RawResourceBearer + SceneFactory
    {
        // Wait for the device to be idle
        unsafe {
            self.render_context.borrow().wait_until_device_idle()?;
        }

        // Get needed things
        let core = self.render_core.borrow();
        let scene = app.get_scene();
        let renderable = scene.get_renderable();

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
                    app,
                    context.get_swapchain_image_count(),
                    current_extent.width,
                    current_extent.height)?;
        }
        self.record_graphics_commands(&renderable)?;
        self.last_known_client_area_size = new_client_area_size;
        Ok(())
    }

    pub fn render_frame<M, A>(&mut self, app: &A) -> Result<PresentResult, VkError> where
        M: 'static + Send + Debug,
        A: 'static + WindowEventHandler<M> + RenderEventHandler + RawResourceBearer + SceneFactory
    {
        let mut context = self.render_context.borrow_mut();
        let resource_manager = self.resource_manager.borrow();
        unsafe {
            let (image_index, up_to_date) = context.acquire_next_image()?;
            if !up_to_date {
                return Ok(PresentResult::SwapchainOutOfDate);
            }

            let scene = app.get_scene();
            let renderable = scene.get_renderable();
            renderable.prepare_frame_render(image_index, &resource_manager)?;
            context.submit_and_present()
        }
    }
}
