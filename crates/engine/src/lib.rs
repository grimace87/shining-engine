
use model::StaticVertex;
use vk::{
    VkCore, VkContext, OffscreenFramebufferWrapper, RenderpassWrapper, PipelineWrapper
};
use window::{RenderEventHandler, WindowEventHandler, Window};
use resource::{ResourceManager, RawResourceBearer, TexturePixelFormat};
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
        unsafe {

            // Creation of required components
            let core = VkCore::new(&window, vec![]).unwrap();
            let mut context = VkContext::new(&core, &window).unwrap();
            let mut resource_manager = ResourceManager::new();
            resource_manager.load_resources_from(&context, &app).unwrap();

            // Create the pipelines
            let (mut framebuffer_1, renderpass_1, mut pipeline_1) =
                Self::create_pipeline(&context, &resource_manager);

            // Release resources
            pipeline_1.destroy_resources(&context);
            renderpass_1.destroy_resources(&context);
            framebuffer_1.destroy(&context).unwrap();
            resource_manager.free_resources(&mut context).unwrap();
        }

        window.run(app);
    }

    unsafe fn create_pipeline(
        context: &VkContext,
        resource_manager: &ResourceManager<VkContext>
    ) -> (OffscreenFramebufferWrapper, RenderpassWrapper, PipelineWrapper) {

        let render_extent = ash::vk::Extent2D::builder()
            .width(128)
            .height(128)
            .build();
        let framebuffer = OffscreenFramebufferWrapper::new(
            context,
            render_extent.width,
            render_extent.height,
            TexturePixelFormat::Rgba,
            TexturePixelFormat::None)
            .unwrap();
        let renderpass = RenderpassWrapper::new_with_offscreen_target(
            context,
            &framebuffer)
            .unwrap();
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
            ash::vk::ShaderStageFlags::VERTEX | ash::vk::ShaderStageFlags::FRAGMENT,
            false,
            TEXTURE_INDEX_TERRAIN,
            false,
            render_extent)
            .unwrap();
        (framebuffer, renderpass, pipeline)
    }
}
