
pub trait ResourceLoader where Self: Sized {
    type LoadError;
    fn get_current_swapchain_extent(&self) -> Result<(u32, u32), Self::LoadError>;
    fn make_error(message: String) -> Self::LoadError;
}
