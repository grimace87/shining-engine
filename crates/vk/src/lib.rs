mod core;
mod context;
mod resource;

pub use crate::core::VkCore;
pub use crate::core::FeatureDeclaration;
pub use context::VkContext;
pub use resource::{
    ImageUsage,
    TexturePixelFormat,
    buffer::BufferWrapper,
    image::ImageWrapper
};

#[derive(Debug)]
pub enum VkError {
    OpFailed(String)
}
