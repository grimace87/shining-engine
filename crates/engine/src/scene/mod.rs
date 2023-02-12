pub mod null;
pub mod stock;

use vk_renderer::{VkError, VkContext};
use resource::{RawResourceBearer, ResourceManager};
use ash::{Device, vk};

pub trait SceneFactory {
    fn get_scene(&self) -> Box<dyn Scene>;
}

pub trait Scene {

    /// Build an object that bears resources
    fn get_resource_bearer(&self) -> Box<dyn RawResourceBearer>;

    /// Record commands once such that they can be executed later once per frame
    unsafe fn record_commands(
        &self,
        device: &Device,
        command_buffer: vk::CommandBuffer,
        render_extent: vk::Extent2D,
        resource_manager: &ResourceManager<VkContext>,
        swapchain_image_index: usize
    ) -> Result<(), VkError>;

    /// Perform per-frame state updates
    fn update(&mut self, time_step_seconds: f64);

    /// Prepare for rendering a frame
    unsafe fn prepare_frame_render(
        &self,
        context: &VkContext,
        swapchain_image_index: usize,
        resource_manager: &ResourceManager<VkContext>
    ) -> Result<(), VkError>;
}
