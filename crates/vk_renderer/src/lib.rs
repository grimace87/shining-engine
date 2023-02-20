mod core;
mod context;
mod mem;
mod resource;
mod pipeline;

pub use crate::core::VkCore;
pub use crate::core::FeatureDeclaration;
pub use context::VkContext;
pub use context::PresentResult;
pub use context::Queue;
pub use crate::resource::{
    ShaderStage, ShaderCreationData, UboUsage, DescriptorSetLayoutCreationData,
    PipelineLayoutCreationData
};
pub use crate::resource::util::{TextureCodec, ResourceUtilities};
pub use crate::resource::buffer::{BufferWrapper, BufferUsage, VboCreationData};
pub use crate::resource::image::{ImageWrapper, ImageUsage, TexturePixelFormat, TextureCreationData};
pub use pipeline::{
    wrapper::{PipelineWrapper, PipelineCreationData},
    renderpass::{RenderpassWrapper, RenderpassTarget, RenderpassCreationData},
    offscreen_framebuffer::{OffscreenFramebufferWrapper, OffscreenFramebufferData}
};

#[derive(Debug)]
pub enum VkError {
    OpFailed(String),
    MissingResource(String),
    Compatibility(String),
    EngineError(String),
    UserError(String)
}
