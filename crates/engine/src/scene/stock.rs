
use crate::Scene;
use camera::PlayerCamera;
use vk_renderer::{
    VkContext, VkError, TextureCodec, ResourceUtilities, RenderpassWrapper, PipelineWrapper,
    BufferWrapper, BufferUsage, ImageUsage, VboCreationData, ShaderCreationData, ShaderStage,
    RenderpassCreationData, DescriptorSetLayoutCreationData, PipelineLayoutCreationData,
    PipelineCreationData, RenderpassTarget, UboUsage, ImageWrapper
};
use model::{StaticVertex, COLLADA, Config};
use resource::{ResourceManager, Handle, RawResourceBearer, Resource};
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

impl Scene<VkContext> for StockScene {

    fn get_resource_bearer(&self) -> Box<dyn RawResourceBearer<VkContext>> {
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

        let renderpass = resource_manager
            .get_item::<RenderpassWrapper>(
                Handle::with_minor_variation(RENDERPASS_INDEX_MAIN, swapchain_image_index as u32)
                    .unwrap())
            .unwrap();
        let pipeline = resource_manager
            .get_item::<PipelineWrapper>(
                Handle::with_minor_variation(PIPELINE_INDEX_MAIN, swapchain_image_index as u32)
                    .unwrap())
            .unwrap();
        let pipeline_layout = resource_manager
            .get_item::<vk::PipelineLayout>(
                Handle::with_unique_id(PIPELINE_LAYOUT_INDEX_MAIN, 0))
            .unwrap();

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
        let vertex_buffer = resource_manager
            .get_item::<BufferWrapper>(
                Handle::with_unique_id(VBO_INDEX_SCENE, 0))
            .unwrap();
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
            vertex_buffer.element_count as u32,
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

    fn update(&mut self, time_step_millis: u64, control_dx: f32, control_dy: f32) {
        let time_step_seconds = (time_step_millis as f64) * 0.001;
        self.total_time = self.total_time + time_step_seconds;
        self.camera.update(time_step_millis, control_dx, control_dy);

        let model_matrix = Matrix4::from_angle_y(Rad(self.total_time as f32));
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
        let pipeline = resource_manager
            .get_item::<PipelineWrapper>(
                Handle::with_minor_variation(PIPELINE_INDEX_MAIN, swapchain_image_index as u32)
                    .unwrap())
            .unwrap();
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

impl RawResourceBearer<VkContext> for StockResourceBearer {

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
        let creation_data = VboCreationData {
            vertex_data: Some(scene_model.vertices.as_ptr() as *const u8),
            vertex_size_bytes: std::mem::size_of::<StaticVertex>(),
            vertex_count: scene_model.vertices.len(),
            draw_indexed: false,
            index_data: None,
            usage: BufferUsage::InitialiseOnceVertexBuffer
        };
        let model = BufferWrapper::create(loader, &manager, &creation_data)?;
        manager.push_new_with_handle(
            Handle::with_unique_id(VBO_INDEX_SCENE, 0),
            model);

        let creation_data = ResourceUtilities::decode_texture(
            TERRAIN_TEXTURE_BYTES,
            TextureCodec::Jpeg,
            ImageUsage::TextureSampleOnly)
            .unwrap();
        let texture = ImageWrapper::create(loader, &manager, &creation_data)?;
        manager.push_new_with_handle(
            Handle::with_unique_id(TEXTURE_INDEX_TERRAIN, 0),
            texture);

        let creation_data = ShaderCreationData {
            data: VERTEX_SHADER,
            stage: ShaderStage::Vertex
        };
        let vertex_shader = vk::ShaderModule::create(loader, &manager, &creation_data)?;
        manager.push_new_with_handle(
            Handle::with_unique_id(SHADER_INDEX_VERTEX, 0),
            vertex_shader);

        let creation_data = ShaderCreationData {
            data: FRAGMENT_SHADER,
            stage: ShaderStage::Fragment
        };
        let fragment_shader = vk::ShaderModule::create(loader, &manager, &creation_data)?;
        manager.push_new_with_handle(
            Handle::with_unique_id(SHADER_INDEX_FRAGMENT, 0),
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
            if let Some(item) = manager.remove_item::<RenderpassWrapper>(
                Handle::with_minor_variation(RENDERPASS_INDEX_MAIN, i as u32).unwrap()
            ) {
                item.release(&loader);
            }
        }

        if let Some(item) = manager.remove_item::<vk::DescriptorSetLayout>(
            Handle::with_unique_id(DESCRIPTOR_SET_LAYOUT_INDEX_MAIN, 0)
        ) {
            item.release(&loader);
        }

        if let Some(item) = manager.remove_item::<vk::PipelineLayout>(
            Handle::with_unique_id(PIPELINE_LAYOUT_INDEX_MAIN, 0)
        ) {
            item.release(&loader);
        }

        for i in 0..swapchain_image_count {
            if let Some(item) = manager.remove_item::<PipelineWrapper>(
                Handle::with_minor_variation(PIPELINE_INDEX_MAIN, i as u32).unwrap()
            ) {
                item.release(&loader);
            }
        }

        for i in 0..swapchain_image_count {
            let creation_data = RenderpassCreationData {
                target: RenderpassTarget::SwapchainImageWithDepth,
                swapchain_image_index: i as usize
            };
            let renderpass = RenderpassWrapper::create(loader, &manager, &creation_data)?;
            manager.push_new_with_handle(
                Handle::with_minor_variation(RENDERPASS_INDEX_MAIN, i as u32)
                    .unwrap(),
                renderpass);
        }

        let creation_data = DescriptorSetLayoutCreationData {
            ubo_usage: UboUsage::VertexShaderRead
        };
        let descriptor_set_layout = vk::DescriptorSetLayout::create(loader, &manager, &creation_data)?;
        manager.push_new_with_handle(
            Handle::with_unique_id(DESCRIPTOR_SET_LAYOUT_INDEX_MAIN, 0),
            descriptor_set_layout);

        let creation_data = PipelineLayoutCreationData {
            descriptor_set_layout_index: DESCRIPTOR_SET_LAYOUT_INDEX_MAIN
        };
        let pipeline_layout = vk::PipelineLayout::create(loader, &manager, &creation_data)?;
        manager.push_new_with_handle(
            Handle::with_unique_id(PIPELINE_LAYOUT_INDEX_MAIN, 0),
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
                ubo_size_bytes: std::mem::size_of::<StockUbo>(),
                swapchain_image_index: i as usize
            };
            let pipeline = PipelineWrapper::create(loader, &manager, &creation_data)?;
            manager.push_new_with_handle(
                Handle::with_minor_variation(PIPELINE_INDEX_MAIN, i as u32)
                    .unwrap(),
                pipeline);
        }

        Ok(())
    }
}
