
use resource::{
    DescriptorSetLayoutCreationData, OffscreenFramebufferData, PipelineCreationData,
    PipelineLayoutCreationData, RawResourceBearer, RenderpassCreationData, ResourceManager,
    ShaderCreationData, TextureCreationData, VboCreationData
};
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

impl Scene for NullScene {

    fn get_resource_bearer(&self) -> Box<dyn RawResourceBearer> {
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

    fn update(&mut self, _time_step_seconds: f64) {}

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

impl RawResourceBearer for NullResourceBearer {

    fn get_model_resource_ids(&self) -> &[u32] { &[] }

    fn get_texture_resource_ids(&self) -> &[u32] { &[] }

    fn get_shader_resource_ids(&self) -> &[u32] { &[] }

    fn get_offscreen_framebuffer_resource_ids(&self) -> &[u32] { &[] }

    fn get_renderpass_resource_ids(&self) -> &[u32] { &[] }

    fn get_descriptor_set_layout_resource_ids(&self) -> &[u32] { &[] }

    fn get_pipeline_layout_resource_ids(&self) -> &[u32] { &[] }

    fn get_pipeline_resource_ids(&self) -> &[u32] { &[] }

    fn get_raw_model_data(&self, _id: u32) -> VboCreationData {
        panic!("No resource data exists");
    }

    fn get_raw_texture_data(&self, _id: u32) -> TextureCreationData {
        panic!("No resource data exists");
    }

    fn get_raw_shader_data(&self, _id: u32) -> ShaderCreationData {
        panic!("No resource data exists");
    }

    fn get_raw_offscreen_framebuffer_data(&self, _id: u32) -> OffscreenFramebufferData {
        panic!("No resource data exists");
    }

    fn get_raw_renderpass_data(&self, _id: u32, _swapchain_image_index: usize) -> RenderpassCreationData {
        panic!("No resource data exists");
    }

    fn get_raw_descriptor_set_layout_data(&self, _id: u32) -> DescriptorSetLayoutCreationData {
        panic!("No resource data exists");
    }

    fn get_raw_pipeline_layout_data(&self, _id: u32) -> PipelineLayoutCreationData {
        panic!("No resource data exists");
    }

    fn get_raw_pipeline_data(&self, _id: u32, _swapchain_image_index: usize) -> PipelineCreationData {
        panic!("No resource data exists");
    }
}
