
use crate::Scene;
use camera::PlayerCamera;
use vk_renderer::{VkContext, VkError, TextureCodec, ResourceUtilities};
use model::{StaticVertex, COLLADA, Config};
use resource::{
    ResourceManager, BufferUsage, ImageUsage, VboCreationData, TextureCreationData,
    RawResourceBearer, ShaderCreationData, ShaderStage, RenderpassCreationData,
    DescriptorSetLayoutCreationData, PipelineLayoutCreationData, PipelineCreationData,
    RenderpassTarget, UboUsage, OffscreenFramebufferData
};
use vk_shader_macros::include_glsl;
use ash::{Device, vk};
use cgmath::{Matrix4, SquareMatrix, Rad};
use std::borrow::Borrow;

const VBO_INDEX_SCENE: u32 = 0;
const SCENE_MODEL_BYTES: &[u8] =
    include_bytes!("../../../../resources/test/models/Cubes.dae");

const TEXTURE_INDEX_TERRAIN: u32 = 0;
const TERRAIN_TEXTURE_BYTES: &[u8] =
    include_bytes!("../../../../resources/test/textures/simple_outdoor_texture.jpg");

const SHADER_INDEX_VERTEX: u32 = 0;
const VERTEX_SHADER: &[u32] = include_glsl!("../../resources/test/shaders/stock.vert");

const SHADER_INDEX_FRAGMENT: u32 = 1;
const FRAGMENT_SHADER: &[u32] = include_glsl!("../../resources/test/shaders/stock.frag");

const RENDERPASS_INDEX_MAIN: u32 = 0;

const DESCRIPTOR_SET_LAYOUT_INDEX_MAIN: u32 = 0;

const PIPELINE_LAYOUT_INDEX_MAIN: u32 = 0;

const PIPELINE_INDEX_MAIN: u32 = 0;

#[repr(C)]
pub struct StockUbo {
    pub mvp_matrix: Matrix4<f32>
}

/// TODO - Replace this type with derived implementations of Renderable using macros or some such.
/// For now, this implementation will assume a basic rendering style that draws a textured model
/// without any explicit lighting.
pub struct StockScene {
    total_time: f64,
    camera: PlayerCamera,
    ubo: StockUbo
}

pub struct StockResourceBearer {}

impl StockScene {
    pub fn new() -> Self {
        Self {
            total_time: 0.0,
            camera: PlayerCamera::new(0.0, 1.5, -5.0, 0.0),
            ubo: StockUbo {
                mvp_matrix: Matrix4::identity()
            }
        }
    }
}

impl Scene for StockScene {

    fn get_resource_bearer(&self) -> Box<dyn RawResourceBearer> {
        Box::new(StockResourceBearer::new())
    }

    /// Stock rendering operation renders directly to the swapchain framebuffer
    /// TODO - Fetch renderpass, framebuffer from the resource manager. Evidently we also need the pipeline, the pipeline layout, and the descriptor set.
    unsafe fn record_commands(
        &self,
        device: &Device,
        command_buffer: vk::CommandBuffer,
        render_extent: vk::Extent2D,
        resource_manager: &ResourceManager<VkContext>,
        swapchain_image_index: usize
    ) -> Result<(), VkError> {

        // Query correct renderpass
        let renderpass_definition = RenderpassCreationData {
            target: RenderpassTarget::SwapchainImageWithDepth,
            swapchain_image_index
        };
        let complex_id = renderpass_definition.encode_complex_renderpass_id(
            0,
            render_extent.width,
            render_extent.height);
        let renderpass = resource_manager
            .get_renderpass_handle(complex_id)?;

        // Query correct pipeline and layout
        let pipeline_definition = PipelineCreationData {
            pipeline_layout_index: 0,
            renderpass_index: 0,
            descriptor_set_layout_id: 0,
            vertex_shader_index: 0,
            fragment_shader_index: 0,
            vbo_index: 0,
            texture_index: 0,
            vbo_stride_bytes: 0,
            ubo_size_bytes: 0,
            swapchain_image_index
        };
        let complex_id = pipeline_definition.encode_complex_pipeline_id(0);
        let pipeline = resource_manager.get_pipeline_handle(complex_id)?;
        let pipeline_layout = resource_manager.get_pipeline_layout_handle(0)?;

        // Begin recording
        let begin_info = vk::CommandBufferBeginInfo::builder();
        device.begin_command_buffer(command_buffer, &begin_info)
            .map_err(|e| VkError::OpFailed(format!("{:?}", e)))?;

        // Begin the renderpass
        let clear_values = [
            vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0.0, 0.3, 0.0, 1.0]
                }
            },
            vk::ClearValue {
                depth_stencil: vk::ClearDepthStencilValue {
                    depth: 1.0,
                    stencil: 0
                }
            }
        ];
        let renderpass_begin_info = vk::RenderPassBeginInfo::builder()
            .render_pass(renderpass.renderpass)
            .framebuffer(renderpass.swapchain_framebuffer)
            .render_area(vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: render_extent
            })
            .clear_values(&clear_values);
        device.cmd_begin_render_pass(
            command_buffer, &renderpass_begin_info, vk::SubpassContents::INLINE);

        // Bind the pipeline and do rendering work
        let (vertex_buffer, vertex_count) = resource_manager.get_vbo_handle(0)?;
        device.cmd_bind_pipeline(
            command_buffer,
            vk::PipelineBindPoint::GRAPHICS,
            pipeline.get_pipeline());
        device.cmd_bind_vertex_buffers(
            command_buffer,
            0,
            &[vertex_buffer.buffer],
            &[0]);
        device.cmd_bind_descriptor_sets(
            command_buffer,
            vk::PipelineBindPoint::GRAPHICS,
            *pipeline_layout,
            0,
            &[pipeline.get_descriptor_set()],
            &[]);
        device.cmd_draw(
            command_buffer,
            vertex_count as u32,
            1,
            0,
            0);

        // End the renderpass
        device.cmd_end_render_pass(command_buffer);

        // End recording
        device.end_command_buffer(command_buffer)
            .map_err(|e| VkError::OpFailed(format!("{:?}", e)))?;
        Ok(())
    }

    fn update(&mut self, time_step_seconds: f64) {
        self.total_time = self.total_time + time_step_seconds;

        let model_matrix = Matrix4::from_angle_y(Rad(self.total_time as f32 * 0.002));
        let view_matrix = self.camera.get_view_matrix();
        let projection_matrix = self.camera.get_projection_matrix();
        self.ubo.mvp_matrix = projection_matrix * view_matrix * model_matrix;
    }

    unsafe fn prepare_frame_render(
        &self,
        context: &VkContext,
        swapchain_image_index: usize,
        resource_manager: &ResourceManager<VkContext>
    ) -> Result<(), VkError> {
        let resource_bearer = self.get_resource_bearer();
        let pipeline_description = resource_bearer
            .get_raw_pipeline_data(0, swapchain_image_index);
        let complex_id = pipeline_description.encode_complex_pipeline_id(0);
        let pipeline = resource_manager.get_pipeline_handle(complex_id)?;
        pipeline.update_uniform_buffer(
            context,
            self.ubo.borrow() as *const StockUbo as *const u8,
            std::mem::size_of::<StockUbo>())?;
        Ok(())
    }
}

impl StockResourceBearer {
    pub fn new() -> Self {
        Self {}
    }
}

impl RawResourceBearer for StockResourceBearer {

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
        ResourceUtilities::decode_texture(
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
            ubo_size_bytes: std::mem::size_of::<StockUbo>(),
            swapchain_image_index
        }
    }
}
