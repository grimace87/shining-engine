pub mod buffer;
pub mod image;
pub mod util;

use crate::BufferWrapper;
use crate::ImageWrapper;
use crate::VkError;
use model::StaticVertex;
use resource::{ResourceLoader, BufferUsage, VboCreationData, TextureCreationData, ShaderCreationData};
use ash::vk;

impl ResourceLoader for crate::VkContext {

    type VertexBufferHandle = BufferWrapper;
    type TextureHandle = ImageWrapper;
    type ShaderHandle = vk::ShaderModule;
    type LoadError = VkError;

    fn load_model(&self, raw_data: &VboCreationData) -> Result<(BufferWrapper, usize), VkError> {
        let buffer = unsafe {
            BufferWrapper::new::<StaticVertex>(
                self,
                BufferUsage::InitialiseOnceVertexBuffer,
                raw_data.vertex_count * std::mem::size_of::<StaticVertex>(), // TODO - different vertex types?
                Some(&raw_data.vertex_data))?
        };
        Ok((buffer, raw_data.vertex_count))
    }

    fn release_model(&mut self, model: &BufferWrapper) -> Result<(), VkError> {
        unsafe {
            let (allocator, _) = self.get_mem_allocator();
            model.destroy(allocator)
        }
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
        unsafe {
            let (allocator, _) = self.get_mem_allocator();
            texture.destroy(&self.device, allocator)
        }
    }

    fn load_shader(&self, raw_data: &ShaderCreationData) -> Result<Self::ShaderHandle, Self::LoadError> {
        unsafe {
            let shader_create_info = vk::ShaderModuleCreateInfo::builder()
                .code(raw_data.data);
            self.device
                .create_shader_module(&shader_create_info, None)
                .map_err(|e| VkError::OpFailed(format!("{:?}", e)))
        }
    }

    fn release_shader(&mut self, shader: &Self::ShaderHandle) -> Result<(), Self::LoadError> {
        unsafe {
            self.device.destroy_shader_module(*shader, None);
        }
        Ok(())
    }

    #[inline]
    fn make_error(message: String) -> Self::LoadError {
        VkError::MissingResource(message)
    }
}
