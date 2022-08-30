mod core;
mod context;

pub use crate::core::VkCore;
pub use crate::core::FeatureDeclaration;
pub use context::VkContext;

#[derive(Debug)]
pub enum VkError {
    OpFailed(String)
}
