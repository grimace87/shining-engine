use crate::Renderable;

use resource::ResourceManager;
use vk_renderer::{VkContext, VkError, RenderpassWrapper, PipelineWrapper};
use ash::vk;

/// TODO - Replace this type with derived implementations of Renderable using macros or some such.
/// For now, this implementation will assume a basic rendering style that draws a textured model
/// without any explicit lighting.
pub struct NullRenderable {}

impl NullRenderable {
    pub fn new() -> Self {
        Self {}
    }
}

impl Renderable for NullRenderable {

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
                0,
                0,
                vk::ShaderStageFlags::VERTEX,
                false,
                0,
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

    fn update(&mut self, _time_step_seconds: f64) {}
}
