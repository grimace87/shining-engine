
/// Test features in the mem module.
/// Creating memory relies on a VkCore and a VkContext, which in turn rely on having an actual
/// window to use for the instance and the surface.
///
/// The test creates a window, creates a VkCore and a VkContext, and then creates a bunch of memory
/// objects. Then it tears everything down.

use vk_renderer::{
    VkCore, VkContext, TextureCodec, ResourceUtilities, BufferUsage, ImageUsage, VboCreationData, BufferWrapper,
    ImageWrapper
};
use window::{
    WindowEventLooper, RenderCycleEvent, RenderEventHandler, ControlFlow, Event, WindowEvent,
    WindowEventHandler, WindowStateEvent, Window, MessageProxy, WindowCommand
};
use model::{COLLADA, Config, StaticVertex};
use ecs::{
    EcsManager, Handle,
    resource::{RawResourceBearer, Resource}
};
use error::EngineError;
use std::fmt::Debug;

const VBO_INDEX_SCENE: u32 = 0;
const SCENE_MODEL_BYTES: &[u8] =
    include_bytes!("../../../resources/test/models/Cubes.dae");

const TEXTURE_INDEX_TERRAIN: u32 = 0;
const TERRAIN_TEXTURE_BYTES: &[u8] =
    include_bytes!("../../../resources/test/textures/simple_outdoor_texture.jpg");

struct ResourceSource {}

impl RawResourceBearer<VkContext> for ResourceSource {

    fn initialise_static_resources(
        &self,
        ecs: &mut EcsManager<VkContext>,
        loader: &VkContext
    ) -> Result<(), EngineError> {

        let scene_model = {
            let collada = COLLADA::new(&SCENE_MODEL_BYTES);
            let mut models = collada.extract_models(Config::default());
            models.remove(0)
        };
        let scene_vertex_count = scene_model.vertices.len();
        let creation_data = VboCreationData {
            vertex_data: Some(scene_model.vertices.as_ptr() as *const u8),
            vertex_size_bytes: std::mem::size_of::<StaticVertex>(),
            vertex_count: scene_vertex_count,
            draw_indexed: false,
            index_data: None,
            usage: BufferUsage::InitialiseOnceVertexBuffer
        };
        let vertex_buffer = BufferWrapper::create(loader, &ecs, &creation_data)?;
        ecs.push_new_with_handle(
            Handle::for_resource(VBO_INDEX_SCENE),
            vertex_buffer);

        let creation_data = ResourceUtilities::decode_texture(
            TERRAIN_TEXTURE_BYTES,
            TextureCodec::Jpeg,
            ImageUsage::TextureSampleOnly)
            .unwrap();
        let texture = ImageWrapper::create(loader, &ecs, &creation_data)?;
        ecs.push_new_with_handle(
            Handle::for_resource(TEXTURE_INDEX_TERRAIN),
            texture);

        Ok(())
    }

    fn reload_dynamic_resources(
        &self,
        _ecs: &mut EcsManager<VkContext>,
        _loader: &mut VkContext,
        _swapchain_image_count: usize
    ) -> Result<(), EngineError> {
        Ok(())
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

            // Creation
            let mut core = VkCore::new(window, vec![]).unwrap();
            let mut context = VkContext::new(&core, window).unwrap();
            let resource_source: Box<dyn RawResourceBearer<VkContext>> = Box::new(ResourceSource {});
            let mut ecs = EcsManager::new();
            resource_source
                .initialise_static_resources(&mut ecs, &context)
                .unwrap();

            // Release
            ecs.free_all_resources(&context).unwrap();
            context.teardown();
            core.teardown();
        }
        Self { message_proxy }
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
    let window = Window::new("Vulkan Mem Test", &looper);
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
