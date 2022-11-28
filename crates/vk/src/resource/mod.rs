pub mod buffer;
pub mod image;
pub mod util;

use crate::BufferWrapper;
use crate::ImageWrapper;
use crate::VkError;
use crate::mem::MemoryUsage;
use model::StaticVertex;
use resource::{ResourceLoader, VboCreationData, TextureCreationData};
use ash::vk;

impl ResourceLoader for crate::VkContext {

    type VertexBufferHandle = BufferWrapper;
    type TextureHandle = ImageWrapper;
    type LoadError = VkError;

    fn load_model(&self, raw_data: &VboCreationData) -> Result<BufferWrapper, VkError> {
        let buffer = unsafe {
            let mut buffer = BufferWrapper::new(
                self.get_mem_allocator(),
                raw_data.vertex_count * std::mem::size_of::<StaticVertex>(), // TODO - different vertex types?
                vk::BufferUsageFlags::VERTEX_BUFFER,
                MemoryUsage::CpuToGpu)?; // TODO - staging buffer?
            buffer.update::<StaticVertex>(
                self.get_mem_allocator(),
                0,
                raw_data.vertex_data.as_ptr(),
                raw_data.vertex_data.len())?;
            buffer
        };
        Ok(buffer)
    }

    fn release_model(&mut self, model: &BufferWrapper) -> Result<(), VkError> {
        unsafe { model.destroy(self.get_mem_allocator()) }
    }

    fn load_texture(&self, raw_data: &TextureCreationData) -> Result<ImageWrapper, VkError> {
        let texture = unsafe {
            match raw_data.layer_data.as_ref() {
                Some(data) => ImageWrapper::new(
                    self,
                    raw_data.usage,
                    raw_data.format,
                    raw_data.width,
                    raw_data.height,
                    Some(data.as_slice()))?,
                // TODO - One per swapchain image?
                None => ImageWrapper::new(
                    self,
                    raw_data.usage,
                    raw_data.format,
                    raw_data.width,
                    raw_data.height,
                    None
                )?
            }
        };
        Ok(texture)
    }

    fn release_texture(&mut self, texture: &ImageWrapper) -> Result<(), VkError> {
        unsafe { texture.destroy(&self.device, self.get_mem_allocator()) }
    }

    #[inline]
    fn make_error(message: String) -> Self::LoadError {
        VkError::MissingResource(message)
    }
}
