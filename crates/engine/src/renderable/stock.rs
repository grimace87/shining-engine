use crate::Renderable;

use model::StaticVertex;
use resource::ResourceManager;
use vk_renderer::{VkContext, VkError, RenderpassWrapper, PipelineWrapper};
use ash::{Device, vk};
use cgmath::{Matrix4, SquareMatrix, Rad};

#[repr(C)]
pub struct CameraUbo {
    pub camera_matrix: Matrix4<f32>
}

/// TODO - Replace this type with derived implementations of Renderable using macros or some such.
/// For now, this implementation will assume a basic rendering style that draws a textured model
/// without any explicit lighting.
pub struct StockRenderable {
    vbo_size_bytes: usize,
    total_time: f64,
    camera_transform: CameraUbo
}

impl StockRenderable {
    pub fn new(vbo_size_bytes: usize) -> Self {
        Self {
            vbo_size_bytes: vbo_size_bytes,
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
                0,
                std::mem::size_of::<StaticVertex>() as u32,
                std::mem::size_of::<CameraUbo>(),
                vk::ShaderStageFlags::VERTEX,
                false,
                0,
                false,
                render_extent
            )?;
        }
        Ok((renderpass, pipeline))
    }

    /// Stock rendering operation renders directly to the swapchain framebuffer
    /// TODO - Fetch renderpass, framebuffer from the resource manager. Evidently we also need the pipeline, the pipeline layout, and the descriptor set.
    unsafe fn record_commands(
        &self,
        device: &Device,
        command_buffer: vk::CommandBuffer,
        render_extent: vk::Extent2D,
        resource_manager: &ResourceManager<VkContext>,
        renderpass: &RenderpassWrapper,
        pipeline: &PipelineWrapper
    ) -> Result<(), VkError> {

        // Begin recording
        let begin_info = vk::CommandBufferBeginInfo::builder();
        device.begin_command_buffer(command_buffer, &begin_info)
            .map_err(|e| VkError::OpFailed(format!("{:?}", e)))?;

        // Begin the renderpass
        let clear_values = [
            vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0.0, 0.0, 0.0, 1.0]
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
            pipeline.get_layout(),
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
        self.camera_transform.camera_matrix = Matrix4::from_angle_y(
            Rad(self.total_time as f32));
    }

    unsafe fn prepare_frame_render(
        &self,
        swapchain_image_index: usize,
        resource_manager: &ResourceManager<VkContext>
    ) -> Result<(), VkError> {
        // TODO - Update uniform buffer if there is one
        Ok(())
    }
}
