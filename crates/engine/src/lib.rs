mod renderable;
mod scene;

pub use renderable::{
    Renderable,
    stock::StockRenderable
};
pub use scene::{Scene, SceneFactory};

use vk_renderer::{VkError, VkCore, VkContext, RenderpassWrapper, PipelineWrapper};
use window::{RenderEventHandler, WindowEventHandler, Window};
use resource::{ResourceManager, RawResourceBearer};
use std::fmt::Debug;

pub struct Engine {}

impl Engine {

    pub fn new() -> Self {
        Self {}
    }

    pub fn run<M, A>(&self, window: Window<M>, app: A) where
        M: 'static + Send + Debug,
        A: 'static + WindowEventHandler<M> + RenderEventHandler + RawResourceBearer + SceneFactory
    {
        // Creation of required components
        let core = unsafe { VkCore::new(&window, vec![]).unwrap() };
        let mut context = VkContext::new(&core, &window).unwrap();
        let mut resource_manager = ResourceManager::new();
        resource_manager.load_resources_from(&context, &app).unwrap();

        // Create the pipelines
        let scene = app.get_scene();
        let renderable = scene.get_renderable();
        let mut pipelines = unsafe {
            Self::create_pipelines(&context, &resource_manager, &renderable).unwrap()
        };

        window.run(app);

        // Release resources
        for (renderpass, pipeline) in pipelines.iter_mut() {
            pipeline.destroy_resources(&context);
            renderpass.destroy_resources(&context);
        }
        resource_manager.free_resources(&mut context).unwrap();
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
}
