
use crate::{
    ResourceManager, VboCreationData, TextureCreationData, ShaderCreationData,
    OffscreenFramebufferData, RenderpassCreationData, DescriptorSetLayoutCreationData,
    PipelineLayoutCreationData, PipelineCreationData
};

pub trait ResourceLoader where Self: Sized {

    type VertexBufferHandle;
    type TextureHandle;
    type ShaderHandle;
    type OffscreenFramebufferHandle;
    type RenderpassHandle;
    type DescriptorSetLayoutHandle;
    type PipelineLayoutHandle;
    type PipelineHandle;
    type LoadError;

    fn get_current_swapchain_extent(&self) -> Result<(u32, u32), Self::LoadError>;

    fn load_model<T: Sized>(
        &self,
        raw_data: &VboCreationData<T>
    ) -> Result<Self::VertexBufferHandle, Self::LoadError>;

    fn load_texture(
        &self,
        raw_data: &TextureCreationData
    ) -> Result<Self::TextureHandle, Self::LoadError>;

    fn load_shader(
        &self,
        raw_data: &ShaderCreationData
    ) -> Result<Self::ShaderHandle, Self::LoadError>;

    fn load_offscreen_framebuffer(
        &self,
        raw_data: &OffscreenFramebufferData
    ) -> Result<Self::OffscreenFramebufferHandle, Self::LoadError>;

    fn load_renderpass(
        &self,
        raw_data: &RenderpassCreationData,
        resource_manager: &ResourceManager<Self>
    ) -> Result<Self::RenderpassHandle, Self::LoadError>;

    fn load_descriptor_set_layout(
        &self,
        raw_data: &DescriptorSetLayoutCreationData
    ) -> Result<Self::DescriptorSetLayoutHandle, Self::LoadError>;

    fn load_pipeline_layout(
        &self,
        raw_data: &PipelineLayoutCreationData,
        resource_manager: &ResourceManager<Self>
    ) -> Result<Self::PipelineLayoutHandle, Self::LoadError>;

    fn load_pipeline(
        &self,
        raw_data: &PipelineCreationData,
        resource_manager: &ResourceManager<Self>,
        swapchain_image_index: usize
    ) -> Result<Self::PipelineHandle, Self::LoadError>;

    fn make_error(message: String) -> Self::LoadError;
}
