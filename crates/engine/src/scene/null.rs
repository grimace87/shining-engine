
use resource::{RawResourceBearer, ResourceManager};
use vk_renderer::{VkContext, VkError};
use ash::{Device, vk};
use crate::Scene;

pub struct NullScene {}

pub struct NullResourceBearer {}

impl NullScene {
    pub fn new() -> Self {
        Self {}
    }
}

impl Scene<VkContext> for NullScene {

    fn get_resource_bearer(&self) -> Box<dyn RawResourceBearer<VkContext>> {
        Box::new(NullResourceBearer::new())
    }

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

    fn update(&mut self, _time_step_millis: u64, _control_dx: f32, _control_dy: f32) {}

    unsafe fn prepare_frame_render(
        &self,
        _context: &VkContext,
        _swapchain_image_index: usize,
        _resource_manager: &ResourceManager<VkContext>
    ) -> Result<(), VkError> {
        Ok(())
    }
}

impl NullResourceBearer {
    pub fn new() -> Self {
        Self {}
    }
}

impl RawResourceBearer<VkContext> for NullResourceBearer {

    fn initialise_static_resources(
        &self,
        _manager: &mut ResourceManager<VkContext>,
        _loader: &VkContext
    ) -> Result<(), VkError> {
        Ok(())
    }

    fn reload_dynamic_resources(
        &self,
        _manager: &mut ResourceManager<VkContext>,
        _loader: &mut VkContext,
        _swapchain_image_count: usize
    ) -> Result<(), VkError> {
        Ok(())
    }
}
