
use model::{StaticVertex, COLLADA, Config};
use engine::{Engine, Renderable, StockRenderable, SceneFactory, Scene};
use resource::{
    RawResourceBearer, VboCreationData, BufferUsage, TextureCreationData, ShaderStage, ImageUsage,
    ShaderCreationData
};
use window::{
    RenderEventHandler, RenderCycleEvent, MessageProxy,
    WindowEventHandler, WindowStateEvent, WindowCommand,
    KeyCode, KeyState
};
use vk_renderer::{TextureCodec, util::decode_texture};
use vk_shader_macros::include_glsl;
use std::fmt::Debug;

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

#[derive(PartialEq, Debug)]
pub enum TestAppMessage {
    RequestQuit
}

struct SimpleScene {}

impl Scene for SimpleScene {
    fn get_renderable(&self) -> Box<dyn Renderable> {
        let vertex_size = std::mem::size_of::<StaticVertex>();
        let vertex_count = 12;
        Box::new(StockRenderable::new(vertex_count * vertex_size))
    }
}

struct QuitsQuicklyApp {
    message_proxy: MessageProxy<WindowCommand<TestAppMessage>>
}

impl QuitsQuicklyApp {
    fn new<T: Send + Debug>(message_proxy: MessageProxy<WindowCommand<TestAppMessage>>) -> Self {
        Self { message_proxy }
    }
}

impl RawResourceBearer for QuitsQuicklyApp {

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

impl WindowEventHandler<TestAppMessage> for QuitsQuicklyApp {

    fn on_window_state_event(&mut self, event: WindowStateEvent) {
        if let WindowStateEvent::KeyEvent(KeyCode::Escape, KeyState::Pressed) = event {
            self.message_proxy.send_event(WindowCommand::RequestClose)
                .unwrap();
        }
    }

    fn on_window_custom_event(&mut self, _event: TestAppMessage) {}
}

impl SceneFactory for QuitsQuicklyApp {
    fn get_scene(&self) -> Box<dyn Scene> {
        Box::new(SimpleScene {})
    }
}

impl RenderEventHandler for QuitsQuicklyApp {

    fn on_render_cycle_event(&self, event: RenderCycleEvent) {
        match event {
            RenderCycleEvent::PrepareUpdate => {
                self.message_proxy.send_event(WindowCommand::RequestRedraw)
                    .unwrap();
            },
            _ => {}
        }
    }
}

// Current setup will intercept a FocusGained state event, then post a custom message.
// This custom message will also be intercepted, at which point a RequestClose command is sent.
fn main() {
    let engine = Engine::<TestAppMessage>::new("Demo App");
    let message_proxy = engine.new_message_proxy();
    let app = QuitsQuicklyApp::new::<WindowCommand<TestAppMessage>>(
        message_proxy.clone());
    engine.run(app);
}
