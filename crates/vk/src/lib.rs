mod core;

pub use crate::core::VkCore;

#[derive(Debug)]
pub enum VkError {
    OpFailed(String)
}
