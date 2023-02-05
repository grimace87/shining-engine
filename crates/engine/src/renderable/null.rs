use crate::Renderable;

use resource::ResourceManager;
use vk_renderer::{VkContext, VkError};
use ash::{Device, vk};

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

    unsafe fn record_commands(
        &self,
        _device: &Device,
        _command_buffer: vk::CommandBuffer,
        _render_extent: vk::Extent2D,
        _resource_manager: &ResourceManager<VkContext>,
        _swapchain_image_index: usize
    ) -> Result<(), VkError> {
        Ok(())
    }

    fn update(&mut self, _time_step_seconds: f64) {}

    unsafe fn prepare_frame_render(
        &self,
        _swapchain_image_index: usize,
        _resource_manager: &ResourceManager<VkContext>
    ) -> Result<(), VkError> {
        Ok(())
    }
}
