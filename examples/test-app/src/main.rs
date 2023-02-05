
use model::{StaticVertex, COLLADA, Config};
use engine::{Engine, Renderable, StockRenderable, SceneFactory, Scene};
use resource::{
    BufferUsage, ImageUsage, VboCreationData, TextureCreationData, RawResourceBearer,
    ShaderCreationData, ShaderStage, RenderpassCreationData,
    DescriptorSetLayoutCreationData, PipelineLayoutCreationData, PipelineCreationData,
    RenderpassTarget, UboUsage, OffscreenFramebufferData
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

const RENDERPASS_INDEX_MAIN: u32 = 0;

const DESCRIPTOR_SET_LAYOUT_INDEX_MAIN: u32 = 0;

const PIPELINE_LAYOUT_INDEX_MAIN: u32 = 0;

const PIPELINE_INDEX_MAIN: u32 = 0;

#[repr(C)]
struct SomeUniformBuffer {
    pub x: f32,
    pub y: f32
}

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

    fn get_offscreen_framebuffer_resource_ids(&self) -> &[u32] {
        &[]
    }

    fn get_renderpass_resource_ids(&self) -> &[u32] {
        &[RENDERPASS_INDEX_MAIN]
    }

    fn get_descriptor_set_layout_resource_ids(&self) -> &[u32] {
        &[DESCRIPTOR_SET_LAYOUT_INDEX_MAIN]
    }

    fn get_pipeline_layout_resource_ids(&self) -> &[u32] {
        &[PIPELINE_LAYOUT_INDEX_MAIN]
    }

    fn get_pipeline_resource_ids(&self) -> &[u32] {
        &[PIPELINE_INDEX_MAIN]
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

    fn get_raw_offscreen_framebuffer_data(&self, _id: u32) -> OffscreenFramebufferData {
        panic!("Bad offscreen framebuffer resource ID");
    }

    fn get_raw_renderpass_data(
        &self,
        id: u32,
        swapchain_image_index: usize
    ) -> RenderpassCreationData {
        if id != RENDERPASS_INDEX_MAIN {
            panic!("Bad renderpass resource ID");
        }
        RenderpassCreationData {
            target: RenderpassTarget::SwapchainImageWithDepth,
            swapchain_image_index
        }
    }

    fn get_raw_descriptor_set_layout_data(&self, id: u32) -> DescriptorSetLayoutCreationData {
        if id != DESCRIPTOR_SET_LAYOUT_INDEX_MAIN {
            panic!("Bad descriptor set layout resource ID");
        }
        DescriptorSetLayoutCreationData {
            ubo_usage: UboUsage::VertexShaderRead
        }
    }

    fn get_raw_pipeline_layout_data(&self, id: u32) -> PipelineLayoutCreationData {
        if id != PIPELINE_LAYOUT_INDEX_MAIN {
            panic!("Bad pipeline layout resource ID");
        }
        PipelineLayoutCreationData {
            descriptor_set_layout_index: DESCRIPTOR_SET_LAYOUT_INDEX_MAIN
        }
    }

    fn get_raw_pipeline_data(
        &self,
        id: u32,
        swapchain_image_index: usize
    ) -> PipelineCreationData {
        if id != PIPELINE_INDEX_MAIN {
            panic!("Bad pipeline resource ID");
        }
        PipelineCreationData {
            pipeline_layout_index: PIPELINE_LAYOUT_INDEX_MAIN,
            renderpass_index: RENDERPASS_INDEX_MAIN,
            descriptor_set_layout_id: DESCRIPTOR_SET_LAYOUT_INDEX_MAIN,
            vertex_shader_index: SHADER_INDEX_VERTEX,
            fragment_shader_index: SHADER_INDEX_FRAGMENT,
            vbo_index: VBO_INDEX_SCENE,
            texture_index: TEXTURE_INDEX_TERRAIN,
            vbo_stride_bytes: std::mem::size_of::<StaticVertex>() as u32,
            ubo_size_bytes: std::mem::size_of::<SomeUniformBuffer>(),
            swapchain_image_index
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
            RenderCycleEvent::PrepareUpdate(_) => {
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
