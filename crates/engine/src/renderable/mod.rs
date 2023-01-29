pub mod null;
pub mod stock;

use ash::{Device, vk};
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
    unsafe fn record_commands(
        &self,
        device: &Device,
        command_buffer: vk::CommandBuffer,
        render_extent: vk::Extent2D,
        resource_manager: &ResourceManager<VkContext>,
        renderpass: &RenderpassWrapper,
        pipeline: &PipelineWrapper
    ) -> Result<(), VkError>;

    /// Perform per-frame state updates
    fn update(&mut self, time_step_seconds: f64);

    /// Prepare for rendering a frame
    unsafe fn prepare_frame_render(
        &self,
        swapchain_image_index: usize,
        resource_manager: &ResourceManager<VkContext>
    ) -> Result<(), VkError>;
}
