
/// Test features in the engine crate.
/// This heavily relies on the Vulkan crate, which is tested more in isolation elsewhere (though it
/// does depend on a window).
///
/// This test creates a more-or-less functioning graphics application.

use engine::{Engine, SceneFactory, Scene, StockScene};
use vk_renderer::{TextureCodec, util::decode_texture};
use window::{
    RenderCycleEvent, RenderEventHandler, WindowEventHandler, WindowStateEvent, WindowCommand
};
use model::{COLLADA, Config, StaticVertex};
use resource::{
    BufferUsage, ImageUsage, VboCreationData, TextureCreationData, RawResourceBearer,
    ShaderCreationData, ShaderStage, RenderpassCreationData,
    DescriptorSetLayoutCreationData, PipelineLayoutCreationData, PipelineCreationData,
    RenderpassTarget, UboUsage, OffscreenFramebufferData
};
use vk_shader_macros::include_glsl;

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

struct EngineTestApp {}

impl RawResourceBearer for EngineTestApp {

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

impl WindowEventHandler<()> for EngineTestApp {
    fn on_window_state_event(&mut self, _event: WindowStateEvent) {}
    fn on_window_custom_event(&mut self, _event: ()) {}
}

impl RenderEventHandler for EngineTestApp {
    fn on_render_cycle_event(&self, _event: RenderCycleEvent) {}
}

impl SceneFactory for EngineTestApp {
    fn get_scene(&self) -> Box<dyn Scene> {
        Box::new(StockScene::new(0))
    }
}

impl EngineTestApp {
    fn new() -> Self {
        Self {}
    }
}

/// Test: send a RequestClose command via the event loop proxy after the window has gained focus.
/// Expected: window opens and then exits very quickly without issue.
fn main() {
    let engine = Engine::<()>::new("Engine Test");
    let message_proxy = engine.new_message_proxy();
    let app = EngineTestApp::new();
    let join_handle = std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(3000));
        message_proxy.send_event(WindowCommand::RequestClose)
            .unwrap();
    });
    engine.run(app);
    join_handle.join().unwrap();
}
