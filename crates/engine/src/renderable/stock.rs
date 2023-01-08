use crate::Renderable;

use model::StaticVertex;
use resource::ResourceManager;
use vk_renderer::{VkContext, VkError, RenderpassWrapper, PipelineWrapper};
use ash::vk;
use cgmath::{Matrix4, SquareMatrix, Rad};
use vk_shader_macros::include_glsl;

const VBO_INDEX_SCENE: u32 = 0;
const SCENE_MODEL_BYTES: &[u8] =
    include_bytes!("../../../../resources/test/models/Cubes.dae");

const TEXTURE_INDEX_TERRAIN: u32 = 0;
const TERRAIN_TEXTURE_BYTES: &[u8] =
    include_bytes!("../../../../resources/test/textures/simple_outdoor_texture.jpg");

const VERTEX_SHADER: &[u32] = include_glsl!("../../resources/test/shaders/simple.vert");

const FRAGMENT_SHADER: &[u32] = include_glsl!("../../resources/test/shaders/simple.frag");

#[repr(C)]
pub struct CameraUbo {
    pub camera_matrix: Matrix4<f32>
}

/// TODO - Replace this type with derived implementations of Renderable using macros or some such.
/// For now, this implementation will assume a basic rendering style that draws a textured model
/// without any explicit lighting.
pub struct StockRenderable {
    vbo_size: usize,
    total_time: f64,
    camera_transform: CameraUbo
}

impl StockRenderable {
    pub fn new(vbo_size_bytes: usize) -> Self {
        Self {
            vbo_size: vbo_size_bytes,
            total_time: 0.0,
            camera_transform: CameraUbo {
                camera_matrix: Matrix4::identity()
            }
        }
    }
}

impl Renderable for StockRenderable {

    fn make_pipeline(
        &self,
        context: &VkContext,
        resource_manager: &ResourceManager<VkContext>,
        swapchain_image_index: usize
    ) -> Result<(RenderpassWrapper, PipelineWrapper), VkError> {
        let render_extent = context.get_extent()?;
        let renderpass = RenderpassWrapper::new_with_swapchain_target(
            context,
            swapchain_image_index)?;
        let mut pipeline = PipelineWrapper::new();
        unsafe {
            pipeline.create_resources(
                context,
                resource_manager,
                &renderpass,
                0,
                1,
                VBO_INDEX_SCENE,
                std::mem::size_of::<StaticVertex>() as u32,
                std::mem::size_of::<CameraUbo>(),
                vk::ShaderStageFlags::VERTEX,
                false,
                TEXTURE_INDEX_TERRAIN,
                false,
                render_extent
            )?;
        }
        Ok((renderpass, pipeline))
    }

    fn record_commands(
        &self,
        command_buffer: vk::CommandBuffer,
        resource_manager: &ResourceManager<VkContext>
    ) {

    }

    fn update(&mut self, time_step_seconds: f64) {
        self.total_time = self.total_time + time_step_seconds;
        self.camera_transform.camera_matrix = Matrix4::from_angle_y(
            Rad(self.total_time as f32));
    }
}
