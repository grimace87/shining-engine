
use crate::{VkContext, VkError, ImageWrapper};
use resource::{TexturePixelFormat, ImageUsage};

/// FramebufferCreationData struct
/// Specification for how a framebuffer (render target) resource is to be created
pub struct OffscreenFramebufferWrapper {
    pub color_texture: ImageWrapper,
    pub depth_texture: Option<ImageWrapper>,
    pub width: u32,
    pub height: u32,
    pub color_format: TexturePixelFormat,
    pub depth_format: TexturePixelFormat
}

impl OffscreenFramebufferWrapper {

    pub unsafe fn new(
        context: &VkContext,
        width: u32,
        height: u32,
        color_format: TexturePixelFormat,
        depth_format: TexturePixelFormat
    ) -> Result<OffscreenFramebufferWrapper, VkError> {
        let color_texture = ImageWrapper::new(
            context,
            ImageUsage::OffscreenRenderSampleColorWriteDepth,
            color_format,
            width,
            height,
            None
        )?;
        let depth_texture = match depth_format {
            TexturePixelFormat::None => None,
            format => Some(
                ImageWrapper::new(
                    context,
                    ImageUsage::DepthBuffer,
                    format,
                    width,
                    height,
                    None
                    )?
            )
        };
        Ok(Self {
            color_texture,
            depth_texture,
            width,
            height,
            color_format,
            depth_format
        })
    }

    pub unsafe fn destroy(&mut self, context: &VkContext) -> Result<(), VkError> {
        let (allocator, _) = context.get_mem_allocator();
        self.color_texture.destroy(&context.device, &allocator)?;
        if let Some(depth_image) = &self.depth_texture {
            depth_image.destroy(&context.device, &allocator)?;
        }
        Ok(())
    }
}
