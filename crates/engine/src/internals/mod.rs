
use crate::{SceneFactory, Renderable};
use vk_renderer::{VkError, VkCore, VkContext, RenderpassWrapper, PipelineWrapper};
use window::{Window, event::{RenderEventHandler, WindowEventHandler}};
use resource::{ResourceManager, RawResourceBearer};
use std::cell::RefCell;
use std::fmt::Debug;

pub struct EngineInternals {
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
        };

        // Initialisation
        Ok(Self {
            render_core: RefCell::new(core),
            render_context: RefCell::new(context),
            resource_manager: RefCell::new(resource_manager),
            renderpasses: RefCell::new(renderpasses),
            pipelines: RefCell::new(pipelines)
        })
    }

    pub fn engine_teardown(&mut self) {

        // Release resources
        for pipeline in self.pipelines.borrow_mut().iter_mut() {
            pipeline.destroy_resources(&self.render_context.borrow());
        }
        for renderpass in self.renderpasses.borrow_mut().iter_mut() {
            renderpass.destroy_resources(&self.render_context.borrow())
        }

        // Free resources
        self.resource_manager.borrow_mut()
            .free_resources(&mut self.render_context.borrow_mut()).unwrap();

        // Destroy renderer
        self.render_context.borrow_mut().teardown();
        self.render_core.borrow_mut().teardown();
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

    pub fn recreate_surface(&mut self) -> Result<(), VkError> {
        todo!()
    }
}
