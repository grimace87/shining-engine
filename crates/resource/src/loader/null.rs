
use crate::ResourceLoader;

pub struct NullResourceLoader;

impl NullResourceLoader {
    pub fn new() -> Self {
        Self
    }
}

impl ResourceLoader for NullResourceLoader {
    type LoadError = String;

    fn get_current_swapchain_extent(&self) -> Result<(u32, u32), String> {
        Ok((1, 1))
    }

    fn make_error(message: String) -> String {
        message
    }
}
