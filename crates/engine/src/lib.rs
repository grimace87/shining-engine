
use model::StaticVertex;
use vk::{
    VkError, VkCore, VkContext, RenderpassWrapper, PipelineWrapper
};
use window::{RenderEventHandler, WindowEventHandler, Window};
use resource::{ResourceManager, RawResourceBearer};
use std::fmt::Debug;
use vk_shader_macros::include_glsl;

const VBO_INDEX_SCENE: u32 = 0;
const SCENE_MODEL_BYTES: &[u8] =
    include_bytes!("../../../resources/test/models/Cubes.dae");

const TEXTURE_INDEX_TERRAIN: u32 = 0;
const TERRAIN_TEXTURE_BYTES: &[u8] =
    include_bytes!("../../../resources/test/textures/simple_outdoor_texture.jpg");

const VERTEX_SHADER: &[u32] = include_glsl!("../../resources/test/shaders/simple.vert");
const FRAGMENT_SHADER: &[u32] = include_glsl!("../../resources/test/shaders/simple.frag");

struct SomeUniformBuffer {
    pub x: f32,
    pub y: f32
}

pub struct Engine {}

impl Engine {

    pub fn new() -> Self {
        Self {}
    }

    pub fn run<M, A>(&self, window: Window<M>, app: A) where
        M: 'static + Send + Debug,
        A: 'static + WindowEventHandler<M> + RenderEventHandler + RawResourceBearer
    {
        // Creation of required components
        let core = unsafe { VkCore::new(&window, vec![]).unwrap() };
        let mut context = VkContext::new(&core, &window).unwrap();
        let mut resource_manager = ResourceManager::new();
        resource_manager.load_resources_from(&context, &app).unwrap();

        // Create the pipelines
        let mut pipelines = unsafe {
            Self::create_pipelines(&context, &resource_manager).unwrap()
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
        resource_manager: &ResourceManager<VkContext>
    ) -> Result<Vec<(RenderpassWrapper, PipelineWrapper)>, VkError> {

        let swapchain_size = context.get_swapchain_image_count();
        let mut pipeline_set = Vec::new();
        for image_index in 0..swapchain_size {
            let render_extent = context.get_extent()?;
            let renderpass = RenderpassWrapper::new_with_swapchain_target(
                context,
                image_index)?;
            let mut pipeline = PipelineWrapper::new();
            pipeline.create_resources(
                context,
                resource_manager,
                &renderpass,
                VERTEX_SHADER,
                FRAGMENT_SHADER,
                VBO_INDEX_SCENE,
                std::mem::size_of::<StaticVertex>() as u32,
                std::mem::size_of::<SomeUniformBuffer>(),
                ash::vk::ShaderStageFlags::VERTEX,
                false,
                TEXTURE_INDEX_TERRAIN,
                false,
                render_extent
            )?;
            pipeline_set.push((renderpass, pipeline));
        }
        Ok(pipeline_set)
    }
}
