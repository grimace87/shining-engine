
/// Test features in the pipeline module.
/// Creating a pipeline relies on a VkCore and a VkContext, which in turn rely on having an actual
/// window to use for the instance and the surface.
///
/// The test creates a window, creates a VkCore and a VkContext, and then creates a some pipeline
/// objects. Then it tears everything down.

use vk_renderer::{
    VkCore, VkContext, VkError, TextureCodec, ResourceUtilities, BufferUsage, ImageUsage,
    VboCreationData, ShaderCreationData, ShaderStage, RenderpassCreationData,
    DescriptorSetLayoutCreationData, PipelineLayoutCreationData, PipelineCreationData,
    RenderpassTarget, UboUsage, BufferWrapper, ImageWrapper, RenderpassWrapper,
    PipelineWrapper
};
use window::{
    WindowEventLooper, RenderCycleEvent, RenderEventHandler, ControlFlow, Event, WindowEvent,
    WindowEventHandler, WindowStateEvent, Window, MessageProxy, WindowCommand
};
use ash::vk;
use std::fmt::Debug;
use vk_shader_macros::include_glsl;

use model::{COLLADA, Config, StaticVertex};
use resource::{ResourceManager, RawResourceBearer, Resource, Handle};

const VBO_INDEX_SCENE: u32 = 0;
const SCENE_MODEL_BYTES: &[u8] =
    include_bytes!("../../../resources/test/models/Cubes.dae");

const TEXTURE_INDEX_TERRAIN: u32 = 0;
const TERRAIN_TEXTURE_BYTES: &[u8] =
    include_bytes!("../../../resources/test/textures/simple_outdoor_texture.jpg");

const SHADER_INDEX_VERTEX: u32 = 0;
const VERTEX_SHADER: &[u32] = include_glsl!("../../resources/test/shaders/stock.vert");

const SHADER_INDEX_FRAGMENT: u32 = 1;
const FRAGMENT_SHADER: &[u32] = include_glsl!("../../resources/test/shaders/stock.frag");

const RENDERPASS_INDEX_MAIN: u32 = 0;

const DESCRIPTOR_SET_LAYOUT_INDEX_MAIN: u32 = 0;

const PIPELINE_LAYOUT_INDEX_MAIN: u32 = 0;

const PIPELINE_INDEX_MAIN: u32 = 0;

#[repr(C)]
struct SomeUniformBuffer {
    pub x: f32,
    pub y: f32
}

struct ResourceSource {}

impl RawResourceBearer<VkContext> for ResourceSource {

    fn initialise_static_resources(
        &self,
        manager: &mut ResourceManager<VkContext>,
        loader: &VkContext
    ) -> Result<(), VkError> {

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
        let vertex_buffer = BufferWrapper::create(loader, &manager, &creation_data)?;
        manager.push_new_with_handle(
            Handle::for_resource(VBO_INDEX_SCENE),
            vertex_buffer);

        let creation_data = ResourceUtilities::decode_texture(
            TERRAIN_TEXTURE_BYTES,
            TextureCodec::Jpeg,
            ImageUsage::TextureSampleOnly)
            .unwrap();
        let texture = ImageWrapper::create(loader, &manager, &creation_data)?;
        manager.push_new_with_handle(
            Handle::for_resource(TEXTURE_INDEX_TERRAIN),
            texture);

        let creation_data = ShaderCreationData {
            data: VERTEX_SHADER,
            stage: ShaderStage::Vertex
        };
        let vertex_shader = vk::ShaderModule::create(loader, &manager, &creation_data)?;
        manager.push_new_with_handle(
            Handle::for_resource(SHADER_INDEX_VERTEX),
            vertex_shader);

        let creation_data = ShaderCreationData {
            data: FRAGMENT_SHADER,
            stage: ShaderStage::Fragment
        };
        let fragment_shader = vk::ShaderModule::create(loader, &manager, &creation_data)?;
        manager.push_new_with_handle(
            Handle::for_resource(SHADER_INDEX_FRAGMENT),
            fragment_shader);

        Ok(())
    }

    fn reload_dynamic_resources(
        &self,
        manager: &mut ResourceManager<VkContext>,
        loader: &mut VkContext,
        swapchain_image_count: usize
    ) -> Result<(), VkError> {

        for i in 0..swapchain_image_count {
            let creation_data = RenderpassCreationData {
                target: RenderpassTarget::SwapchainImageWithDepth,
                swapchain_image_index: i
            };
            let renderpass = RenderpassWrapper::create(loader, &manager, &creation_data)?;
            manager.push_new_with_handle(
                Handle::for_resource_variation(RENDERPASS_INDEX_MAIN, i as u32)
                    .unwrap(),
                renderpass);
        }

        let creation_data = DescriptorSetLayoutCreationData {
            ubo_usage: UboUsage::VertexShaderRead
        };
        let descriptor_set_layout = vk::DescriptorSetLayout::create(loader, &manager, &creation_data)?;
        manager.push_new_with_handle(
            Handle::for_resource(DESCRIPTOR_SET_LAYOUT_INDEX_MAIN),
            descriptor_set_layout);

        let creation_data = PipelineLayoutCreationData {
            descriptor_set_layout_index: DESCRIPTOR_SET_LAYOUT_INDEX_MAIN
        };
        let pipeline_layout = vk::PipelineLayout::create(loader, &manager, &creation_data)?;
        manager.push_new_with_handle(
            Handle::for_resource(PIPELINE_LAYOUT_INDEX_MAIN),
            pipeline_layout);

        for i in 0..swapchain_image_count {
            let creation_data = PipelineCreationData {
                pipeline_layout_index: PIPELINE_LAYOUT_INDEX_MAIN,
                renderpass_index: RENDERPASS_INDEX_MAIN,
                descriptor_set_layout_id: DESCRIPTOR_SET_LAYOUT_INDEX_MAIN,
                vertex_shader_index: SHADER_INDEX_VERTEX,
                fragment_shader_index: SHADER_INDEX_FRAGMENT,
                vbo_index: VBO_INDEX_SCENE,
                texture_index: TEXTURE_INDEX_TERRAIN,
                vbo_stride_bytes: std::mem::size_of::<StaticVertex>() as u32,
                ubo_size_bytes: std::mem::size_of::<SomeUniformBuffer>(),
                swapchain_image_index: i
            };
            let pipeline = PipelineWrapper::create(loader, &manager, &creation_data)?;
            manager.push_new_with_handle(
                Handle::for_resource_variation(PIPELINE_INDEX_MAIN, i as u32)
                    .unwrap(),
                pipeline);
        }

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

            // Creation of required components
            let mut core = VkCore::new(window, vec![]).unwrap();
            let mut context = VkContext::new(&core, window).unwrap();
            let resource_source: Box<dyn RawResourceBearer<VkContext>> = Box::new(ResourceSource {});
            let mut resource_manager = ResourceManager::new();
            let swapchain_image_count = context.get_swapchain_image_count();
            resource_source
                .initialise_static_resources(&mut resource_manager, &context)
                .unwrap();
            resource_source
                .reload_dynamic_resources(
                    &mut resource_manager,
                    &mut context,
                    swapchain_image_count)
                .unwrap();

            // Release
            resource_manager.free_all_resources(&context).unwrap();
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
