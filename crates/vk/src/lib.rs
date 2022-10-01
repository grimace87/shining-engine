mod core;
mod context;
mod mem;
mod resource;
mod pipeline;

pub use crate::core::VkCore;
pub use crate::core::FeatureDeclaration;
pub use context::VkContext;
pub use context::Queue;
pub use crate::resource::util::TextureCodec;
pub use crate::resource::buffer::BufferWrapper;
pub use crate::resource::image::ImageWrapper;
pub use pipeline::{
    PipelineWrapper,
    renderpass::RenderpassWrapper,
    offscreen_framebuffer::OffscreenFramebufferWrapper
};

#[derive(Debug)]
pub enum VkError {
    OpFailed(String),
    MissingResource(String),
    Compatibility(String)
}

pub mod util {
    pub use super::resource::util::{
        decode_model,
        decode_texture
    };
}
