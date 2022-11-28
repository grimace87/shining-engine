mod core;
mod context;
mod mem;
mod resource;

pub use crate::core::VkCore;
pub use crate::core::FeatureDeclaration;
pub use context::VkContext;
pub use crate::resource::buffer::BufferWrapper;
pub use crate::resource::image::ImageWrapper;

#[derive(Debug)]
pub enum VkError {
    OpFailed(String),
    MissingResource(String)
}
