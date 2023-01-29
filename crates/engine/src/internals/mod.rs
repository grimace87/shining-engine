
use crate::{SceneFactory, Renderable, StockTimer, Timer};
use vk_renderer::{VkError, VkCore, VkContext, RenderpassWrapper, PipelineWrapper};
use window::{Window, PhysicalSize, event::{RenderEventHandler, WindowEventHandler}};
use resource::{ResourceManager, RawResourceBearer};
use std::cell::RefCell;
use std::fmt::Debug;

pub struct EngineInternals {
    timer: StockTimer,
    last_known_client_area_size: PhysicalSize<u32>,
    render_core: RefCell<VkCore>,
    render_context: RefCell<VkContext>,
    resource_manager: RefCell<ResourceManager<VkContext>>,
    renderpasses: RefCell<Vec<RenderpassWrapper>>,
    pipelines: RefCell<Vec<PipelineWrapper>>
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
        resource_manager.load_resources_from(&context, app).unwrap();

        // Create the pipelines
        let scene = app.get_scene();
        let renderable = scene.get_renderable();
        let mut renderpasses = vec![];
        let mut pipelines = vec![];
        unsafe {
            let new_pipelines = Self::create_pipelines(&context, &resource_manager, &renderable).unwrap();
            for (renderpass, pipeline) in new_pipelines.into_iter() {
                renderpasses.push(renderpass);
                pipelines.push(pipeline);
            }
            // TODO - record command buffers
        };

        // Initialisation
        Ok(Self {
            timer: StockTimer::new(),
            last_known_client_area_size: PhysicalSize::default(),
            render_core: RefCell::new(core),
            render_context: RefCell::new(context),
            resource_manager: RefCell::new(resource_manager),
            renderpasses: RefCell::new(renderpasses),
            pipelines: RefCell::new(pipelines)
        })
    }

    pub fn engine_teardown(&mut self) {

        // Release resources
        self.destroy_pipelines();

        // Free resources
        self.resource_manager.borrow_mut()
            .free_resources(&mut self.render_context.borrow_mut()).unwrap();

        // Destroy renderer
        self.render_context.borrow_mut().teardown();
        self.render_core.borrow_mut().teardown();
    }

    pub fn pull_time_step_millis(&mut self) -> u64 {
        self.timer.pull_time_step_millis()
    }

    unsafe fn create_pipelines(
        context: &VkContext,
        resource_manager: &ResourceManager<VkContext>,
        renderable: &Box<dyn Renderable>
    ) -> Result<Vec<(RenderpassWrapper, PipelineWrapper)>, VkError> {
        let swapchain_size = context.get_swapchain_image_count();
        let mut pipeline_set = Vec::new();
        for image_index in 0..swapchain_size {
            let (renderpass, pipeline) = renderable.make_pipeline(
                context,
                resource_manager,
                image_index)?;
            pipeline_set.push((renderpass, pipeline));
        }
        Ok(pipeline_set)
    }

    fn destroy_pipelines(&self) {

        let mut pipelines = self.pipelines.borrow_mut();
        for pipeline in pipelines.iter_mut() {
            pipeline.destroy_resources(&self.render_context.borrow());
        }
        pipelines.clear();

        let mut renderpasses = self.renderpasses.borrow_mut();
        for renderpass in renderpasses.iter_mut() {
            renderpass.destroy_resources(&self.render_context.borrow())
        }
        renderpasses.clear();
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

        // Tear down invalidated resources
        self.destroy_pipelines();

        // Get needed things
        let core = self.render_core.borrow();
        let mut context = self.render_context.borrow_mut();
        let resource_manager = self.resource_manager.borrow();
        let mut renderpasses = self.renderpasses.borrow_mut();
        let mut pipelines = self.pipelines.borrow_mut();
        let scene = app.get_scene();
        let renderable = scene.get_renderable();

        // Recreate everything
        unsafe {
            context.recreate_surface(&core, window)?;
            context.regenerate_graphics_command_buffers()?;
            let new_pipelines = Self::create_pipelines(&context, &resource_manager, &renderable).unwrap();
            for (renderpass, pipeline) in new_pipelines.into_iter() {
                renderpasses.push(renderpass);
                pipelines.push(pipeline);
            }
            // TODO - record command buffers
        }
        self.last_known_client_area_size = new_client_area_size;
        Ok(())
    }
}
