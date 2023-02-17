
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

    fn load_model<T: Sized>(
        &self,
        raw_data: &VboCreationData<T>
    ) -> Result<(Self::VertexBufferHandle, usize), Self::LoadError>;
    fn release_model(
        &mut self,
        model: &Self::VertexBufferHandle
    ) -> Result<(), Self::LoadError>;

    fn load_texture(
        &self,
        raw_data: &TextureCreationData
    ) -> Result<Self::TextureHandle, Self::LoadError>;
    fn release_texture(
        &mut self,
        texture: &Self::TextureHandle
    ) -> Result<(), Self::LoadError>;

    fn load_shader(
        &self,
        raw_data: &ShaderCreationData
    ) -> Result<Self::ShaderHandle, Self::LoadError>;
    fn release_shader(
        &mut self,
        shader: &Self::ShaderHandle
    ) -> Result<(), Self::LoadError>;

    fn load_offscreen_framebuffer(
        &self,
        raw_data: &OffscreenFramebufferData
    ) -> Result<Self::OffscreenFramebufferHandle, Self::LoadError>;
    fn release_offscreen_framebuffer(
        &mut self,
        framebuffer: &Self::OffscreenFramebufferHandle
    ) -> Result<(), Self::LoadError>;

    fn load_renderpass(
        &self,
        raw_data: &RenderpassCreationData,
        resource_manager: &ResourceManager<Self>
    ) -> Result<Self::RenderpassHandle, Self::LoadError>;
    fn release_renderpass(
        &mut self,
        renderpass: &Self::RenderpassHandle
    ) -> Result<(), Self::LoadError>;

    fn load_descriptor_set_layout(
        &self,
        raw_data: &DescriptorSetLayoutCreationData
    ) -> Result<Self::DescriptorSetLayoutHandle, Self::LoadError>;
    fn release_descriptor_set_layout(
        &mut self,
        descriptor_set_layout: &Self::DescriptorSetLayoutHandle
    ) -> Result<(), Self::LoadError>;

    fn load_pipeline_layout(
        &self,
        raw_data: &PipelineLayoutCreationData,
        resource_manager: &ResourceManager<Self>
    ) -> Result<Self::PipelineLayoutHandle, Self::LoadError>;
    fn release_pipeline_layout(
        &mut self,
        pipeline_layout: &Self::PipelineLayoutHandle
    ) -> Result<(), Self::LoadError>;

    fn load_pipeline(
        &self,
        raw_data: &PipelineCreationData,
        resource_manager: &ResourceManager<Self>,
        current_swapchain_width: u32,
        current_swapchain_height: u32
    ) -> Result<Self::PipelineHandle, Self::LoadError>;
    fn release_pipeline(
        &mut self,
        pipeline: &Self::PipelineHandle
    ) -> Result<(), Self::LoadError>;

    fn make_error(message: String) -> Self::LoadError;
}
