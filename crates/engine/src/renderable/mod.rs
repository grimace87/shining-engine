pub mod stock;

use ash::vk;
use resource::ResourceManager;
use vk_renderer::{VkContext, VkError, RenderpassWrapper, PipelineWrapper};

pub trait Renderable {

    fn make_pipeline(
        &self,
        context: &VkContext,
        resource_manager: &ResourceManager<VkContext>,
        swapchain_image_index: usize
    ) -> Result<(RenderpassWrapper, PipelineWrapper), VkError>;

    /// Record commands once such that they can be executed later once per frame
    fn record_commands(
        &self,
        command_buffer: vk::CommandBuffer,
        resource_manager: &ResourceManager<VkContext>);

    /// Perform per-frame state updates
    fn update(&mut self, time_step_seconds: f64);
}
