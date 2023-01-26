
/// Test features in the pipeline module.
/// Creating a pipeline relies on a VkCore and a VkContext, which in turn rely on having an actual
/// window to use for the instance and the surface.
///
/// The test creates a window, creates a VkCore and a VkContext, and then creates a some pipeline
/// objects. Then it tears everything down.

use vk_renderer::{
    VkCore, VkContext, OffscreenFramebufferWrapper, RenderpassWrapper, PipelineWrapper,
    TextureCodec, util::decode_texture
};
use window::{
    WindowEventLooper, RenderCycleEvent, RenderEventHandler, ControlFlow, Event, WindowEvent,
    WindowEventHandler, WindowStateEvent, Window, MessageProxy, WindowCommand
};
use std::fmt::Debug;
use vk_shader_macros::include_glsl;

use model::{COLLADA, Config, StaticVertex};
use resource::{
    ResourceManager, BufferUsage, ImageUsage, VboCreationData, TextureCreationData,
    RawResourceBearer, ShaderCreationData, ShaderStage, TexturePixelFormat
};

const VBO_INDEX_SCENE: u32 = 0;
const SCENE_MODEL_BYTES: &[u8] =
    include_bytes!("../../../resources/test/models/Cubes.dae");

const TEXTURE_INDEX_TERRAIN: u32 = 0;
const TERRAIN_TEXTURE_BYTES: &[u8] =
    include_bytes!("../../../resources/test/textures/simple_outdoor_texture.jpg");

const SHADER_INDEX_VERTEX: u32 = 0;
const VERTEX_SHADER: &[u32] = include_glsl!("../../resources/test/shaders/simple.vert");

const SHADER_INDEX_FRAGMENT: u32 = 1;
const FRAGMENT_SHADER: &[u32] = include_glsl!("../../resources/test/shaders/simple.frag");

#[repr(C)]
struct SomeUniformBuffer {
    pub x: f32,
    pub y: f32
}

struct ResourceSource {}

impl RawResourceBearer for ResourceSource {

    fn get_model_resource_ids(&self) -> &[u32] {
        &[VBO_INDEX_SCENE]
    }

    fn get_texture_resource_ids(&self) -> &[u32] {
        &[TEXTURE_INDEX_TERRAIN]
    }

    fn get_shader_resource_ids(&self) -> &[u32] {
        &[SHADER_INDEX_VERTEX, SHADER_INDEX_FRAGMENT]
    }

    fn get_raw_model_data(&self, id: u32) -> VboCreationData {
        if id != VBO_INDEX_SCENE {
            panic!("Bad model resource ID");
        }
        let scene_model = {
            let collada = COLLADA::new(&SCENE_MODEL_BYTES);
            let mut models = collada.extract_models(Config::default());
            models.remove(0)
        };
        let scene_vertex_count = scene_model.vertices.len();
        VboCreationData {
            vertex_data: scene_model.vertices,
            vertex_count: scene_vertex_count,
            draw_indexed: false,
            index_data: None,
            usage: BufferUsage::InitialiseOnceVertexBuffer
        }
    }

    fn get_raw_texture_data(&self, id: u32) -> TextureCreationData {
        if id != TEXTURE_INDEX_TERRAIN {
            panic!("Bad texture resource ID");
        }
        decode_texture(
            TERRAIN_TEXTURE_BYTES,
            TextureCodec::Jpeg,
            ImageUsage::TextureSampleOnly)
            .unwrap()
    }

    fn get_raw_shader_data(&self, id: u32) -> ShaderCreationData {
        match id {
            SHADER_INDEX_VERTEX => ShaderCreationData {
                data: VERTEX_SHADER,
                stage: ShaderStage::Vertex
            },
            SHADER_INDEX_FRAGMENT => ShaderCreationData {
                data: FRAGMENT_SHADER,
                stage: ShaderStage::Fragment
            },
            _ => panic!("Bad texture resource ID")
        }
    }
}

struct VulkanTestApp {
    message_proxy: MessageProxy<WindowCommand<()>>
}

impl VulkanTestApp {

    fn new<T: Send + Debug>(
        window: &Window,
        message_proxy: MessageProxy<WindowCommand<()>>
    ) -> Self {
        unsafe {

            // Creation of required components
            let mut core = VkCore::new(window, vec![]).unwrap();
            let mut context = VkContext::new(&core, window).unwrap();
            let resource_source = ResourceSource {};
            let mut resource_manager = ResourceManager::new();
            resource_manager.load_resources_from(&context, &resource_source).unwrap();

            // Create the pipelines
            let (mut framebuffer_1, renderpass_1, pipeline_1) =
                Self::create_pipeline(&context, &resource_manager);

            // Release
            pipeline_1.destroy_resources(&context);
            renderpass_1.destroy_resources(&context);
            framebuffer_1.destroy(&context).unwrap();
            resource_manager.free_resources(&mut context).unwrap();
            context.teardown();
            core.teardown();
        }
        Self { message_proxy }
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
            SHADER_INDEX_VERTEX,
            SHADER_INDEX_FRAGMENT,
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

impl WindowEventHandler<()> for VulkanTestApp {

    fn on_window_state_event(&mut self, event: WindowStateEvent) {
        if event == WindowStateEvent::FocusGained {
            self.message_proxy.send_event(WindowCommand::RequestClose)
                .unwrap();
        }
    }

    fn on_window_custom_event(&mut self, _event: ()) {}
}

impl RenderEventHandler for VulkanTestApp {
    fn on_render_cycle_event(&self, _event: RenderCycleEvent) {}
}

/// Test: send a RequestClose command via the event loop proxy after the window has gained focus.
/// Expected: window opens and then exits very quickly without issue.
fn main() {
    let looper = WindowEventLooper::<()>::new();
    let message_proxy = looper.create_proxy();
    let window = Window::new("Vulkan Pipeline Test", &looper);
    let mut app = VulkanTestApp::new::<()>(&window, message_proxy.clone());
    let running_window_id = window.get_window_id();
    let _code = looper.run_loop(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
            Event::UserEvent(command) => {
                match command {
                    WindowCommand::RequestClose => {
                        *control_flow = ControlFlow::Exit
                    },
                    WindowCommand::RequestRedraw => {
                        window.request_redraw();
                    },
                    WindowCommand::Custom(e) => {
                        app.on_window_custom_event(e);
                        ()
                    }
                }
            },
            Event::WindowEvent { event, window_id }
            if window_id == running_window_id => {
                match event {
                    WindowEvent::Focused(focused) => {
                        match focused {
                            true => app.on_window_state_event(WindowStateEvent::FocusGained),
                            false => app.on_window_state_event(WindowStateEvent::FocusLost)
                        };
                    },
                    WindowEvent::CloseRequested => {
                        app.on_window_state_event(WindowStateEvent::Closing);
                        *control_flow = ControlFlow::Exit;
                    },
                    _ => {}
                };
            },
            _ => ()
        }
    });
}
