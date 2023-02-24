pub mod null;
pub mod stock;

use vk_renderer::{VkError, VkContext};
use ecs::{EcsManager, resource::{ResourceLoader, RawResourceBearer}};
use ash::{Device, vk};

pub trait SceneFactory<L: ResourceLoader> {
    fn get_scene(&self) -> Box<dyn Scene<L>>;
}

pub trait Scene<L: ResourceLoader> {

    /// Build an object that bears resources
    fn get_resource_bearer(&self) -> Box<dyn RawResourceBearer<L>>;

    /// Record commands once such that they can be executed later once per frame
    unsafe fn record_commands(
        &self,
        device: &Device,
        command_buffer: vk::CommandBuffer,
        render_extent: vk::Extent2D,
        ecs: &EcsManager<L>,
        swapchain_image_index: usize
    ) -> Result<(), VkError>;

    /// Perform per-frame state updates
    fn update(&mut self, time_step_millis: u64, control_dx: f32, control_dy: f32);

    /// Prepare for rendering a frame
    unsafe fn prepare_frame_render(
        &self,
        context: &VkContext,
        swapchain_image_index: usize,
        ecs: &EcsManager<L>
    ) -> Result<(), VkError>;
}
