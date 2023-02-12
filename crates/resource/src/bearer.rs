
use crate::{
    VboCreationData, TextureCreationData, ShaderCreationData, OffscreenFramebufferData,
    RenderpassCreationData, DescriptorSetLayoutCreationData, PipelineLayoutCreationData,
    PipelineCreationData
};

pub trait RawResourceBearer {

    fn get_model_resource_ids(&self) -> &[u32];
    fn get_texture_resource_ids(&self) -> &[u32];
    fn get_shader_resource_ids(&self) -> &[u32];
    fn get_offscreen_framebuffer_resource_ids(&self) -> &[u32];
    fn get_renderpass_resource_ids(&self) -> &[u32];
    fn get_descriptor_set_layout_resource_ids(&self) -> &[u32];
    fn get_pipeline_layout_resource_ids(&self) -> &[u32];
    fn get_pipeline_resource_ids(&self) -> &[u32];

    fn get_raw_model_data(&self, id: u32) -> VboCreationData;
    fn get_raw_texture_data(&self, id: u32) -> TextureCreationData;
    fn get_raw_shader_data(&self, id: u32) -> ShaderCreationData;
    fn get_raw_offscreen_framebuffer_data(&self, id: u32) -> OffscreenFramebufferData;
    fn get_raw_renderpass_data(
        &self, id: u32, swapchain_image_index: usize) -> RenderpassCreationData;
    fn get_raw_descriptor_set_layout_data(&self, id: u32) -> DescriptorSetLayoutCreationData;
    fn get_raw_pipeline_layout_data(&self, id: u32) -> PipelineLayoutCreationData;
    fn get_raw_pipeline_data(
        &self, id: u32, swapchain_image_index: usize) -> PipelineCreationData;
}
