
/// Test features in the engine crate.
/// This heavily relies on the Vulkan crate, which is tested more in isolation elsewhere (though it
/// does depend on a window).
///
/// This test creates a more-or-less functioning graphics application.

use engine::Engine;
use vk_renderer::{TextureCodec, util::decode_texture};
use window::{
    RenderCycleEvent, RenderEventHandler, WindowEventHandler, WindowStateEvent, Window,
    MessageProxy, WindowCommand
};
use model::{COLLADA, Config};
use resource::{
    BufferUsage, ImageUsage, VboCreationData, TextureCreationData, RawResourceBearer
};

const VBO_INDEX_SCENE: u32 = 0;
const SCENE_MODEL_BYTES: &[u8] =
    include_bytes!("../../../resources/test/models/Cubes.dae");

const TEXTURE_INDEX_TERRAIN: u32 = 0;
const TERRAIN_TEXTURE_BYTES: &[u8] =
    include_bytes!("../../../resources/test/textures/simple_outdoor_texture.jpg");

struct EngineTestApp {
    message_proxy: MessageProxy<WindowCommand<()>>
}

impl RawResourceBearer for EngineTestApp {

    fn get_model_resource_ids(&self) -> &[u32] {
        &[VBO_INDEX_SCENE]
    }

    fn get_texture_resource_ids(&self) -> &[u32] {
        &[TEXTURE_INDEX_TERRAIN]
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
            panic!("Bad model resource ID");
        }
        decode_texture(
            TERRAIN_TEXTURE_BYTES,
            TextureCodec::Jpeg,
            ImageUsage::TextureSampleOnly)
            .unwrap()
    }
}

impl WindowEventHandler<()> for EngineTestApp {

    fn on_window_state_event(&mut self, event: WindowStateEvent) {
        if event == WindowStateEvent::FocusGained {
            self.message_proxy.send_event(WindowCommand::RequestClose)
                .unwrap();
        }
    }

    fn on_window_custom_event(&mut self, _event: ()) {}
}

impl RenderEventHandler for EngineTestApp {
    fn on_render_cycle_event(&self, _event: RenderCycleEvent) {}
}

impl EngineTestApp {
    fn new(message_proxy: MessageProxy<WindowCommand<()>>) -> Self {
        Self { message_proxy }
    }
}

/// Test: send a RequestClose command via the event loop proxy after the window has gained focus.
/// Expected: window opens and then exits very quickly without issue.
fn main() {
    let window = Window::<()>::new("Vulkan Core Test");
    let message_proxy = window.new_message_proxy();
    let app = EngineTestApp::new(message_proxy.clone());
    let engine = Engine::new();
    engine.run(window, app);
}
