
use crate::{VkContext, VkError, ImageWrapper, TexturePixelFormat, ImageUsage};
use ecs::{EcsManager, resource::Resource};

/// OffscreenFramebufferData struct
/// Information needed to prepare a non-swapchain framebuffer
pub struct OffscreenFramebufferData {
    pub width: u32,
    pub height: u32,
    pub color_format: TexturePixelFormat,
    pub depth_format: TexturePixelFormat
}

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

impl Resource<VkContext> for OffscreenFramebufferWrapper {
    type CreationData = OffscreenFramebufferData;

    fn create(
        loader: &VkContext,
        _ecs: &EcsManager<VkContext>,
        data: &OffscreenFramebufferData
    ) -> Result<Self, VkError> {
        let framebuffer = unsafe {
            OffscreenFramebufferWrapper::new(
                loader,
                data.width,
                data.height,
                data.color_format,
                data.depth_format)?
        };
        Ok(framebuffer)
    }

    fn release(&self, loader: &VkContext) {
        self.color_texture.release(loader);
        if let Some(depth_image) = &self.depth_texture {
            depth_image.release(loader);
        }
    }
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
}
