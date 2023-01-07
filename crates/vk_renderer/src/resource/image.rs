
use crate::{
    VkError,
    context::VkContext,
    mem::{MemoryAllocator, MemoryAllocation, ManagesImageMemory}
};
use resource::{ImageUsage, TexturePixelFormat};
use ash::{
    vk,
    Device
};

/// ImageCreationParams struct
/// Description for creating an image; should cover all use cases needed by the engine
struct ImageCreationParams {
    format: vk::Format,
    usage: vk::ImageUsageFlags,
    aspect: vk::ImageAspectFlags,
    view_type: vk::ImageViewType,
    initialising_layout: vk::ImageLayout,
    expected_layout: vk::ImageLayout,
    layer_count: u32,
    host_visible: bool
}

/// ImageWrapper struct
/// Wraps a Vulkan image, image view, the format used by the image, and the memory allocation
/// backing the image
pub struct ImageWrapper {
    allocation: MemoryAllocation,
    pub image: vk::Image,
    pub image_view: vk::ImageView,
    pub format: vk::Format
}

impl ImageWrapper {

    /// Create a new instance with nothing useful in it
    pub fn empty() -> ImageWrapper {
        ImageWrapper {
            allocation: MemoryAllocation::null(),
            image: vk::Image::null(),
            image_view: vk::ImageView::null(),
            format: vk::Format::UNDEFINED
        }
    }

    /// Create a new instance, fully initialised
    pub unsafe fn new(
        context: &VkContext,
        usage: ImageUsage,
        format: TexturePixelFormat,
        width: u32,
        height: u32,
        init_layer_data: Option<&[Vec<u8>]>
    ) -> Result<ImageWrapper, VkError> {

        let creation_params = match (usage, format) {
            // Typical depth buffer
            (ImageUsage::DepthBuffer, TexturePixelFormat::Unorm16) => {
                if init_layer_data.is_some() {
                    return Err(VkError::OpFailed(
                        String::from("Initialising depth buffer not allowed")));
                }
                ImageCreationParams {
                    format: vk::Format::D16_UNORM,
                    usage: vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
                    aspect: vk::ImageAspectFlags::DEPTH,
                    view_type: vk::ImageViewType::TYPE_2D,
                    initialising_layout: vk::ImageLayout::UNDEFINED,
                    expected_layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
                    layer_count: 1,
                    host_visible: false
                }
            },

            // Typical off-screen-rendered color attachment
            (ImageUsage::OffscreenRenderSampleColorWriteDepth, TexturePixelFormat::Rgba) => {
                if init_layer_data.is_some() {
                    return Err(VkError::OpFailed(
                        String::from("Initialising off-screen render image not allowed")));
                }
                ImageCreationParams {
                    format: vk::Format::R8G8B8A8_UNORM,
                    usage: vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::COLOR_ATTACHMENT,
                    aspect: vk::ImageAspectFlags::COLOR,
                    view_type: vk::ImageViewType::TYPE_2D,
                    initialising_layout: vk::ImageLayout::UNDEFINED,
                    expected_layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                    layer_count: 1,
                    host_visible: false
                }
            },

            // Typical off-screen-rendered depth attachment
            (ImageUsage::OffscreenRenderSampleColorWriteDepth, TexturePixelFormat::Unorm16) => {
                if init_layer_data.is_some() {
                    return Err(VkError::OpFailed(
                        String::from("Initialising off-screen render image not allowed")));
                }
                ImageCreationParams {
                    format: vk::Format::D16_UNORM,
                    usage: vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
                    aspect: vk::ImageAspectFlags::DEPTH,
                    view_type: vk::ImageViewType::TYPE_2D,
                    initialising_layout: vk::ImageLayout::UNDEFINED,
                    expected_layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                    layer_count: 1,
                    host_visible: false
                }
            },

            // Typical initialised texture
            (ImageUsage::TextureSampleOnly, TexturePixelFormat::Rgba) => {
                if init_layer_data.is_none() {
                    return Err(VkError::OpFailed(
                        String::from("Not initialising sample-only texture not allowed")));
                }
                ImageCreationParams {
                    format: vk::Format::R8G8B8A8_UNORM,
                    usage: vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED,
                    aspect: vk::ImageAspectFlags::COLOR,
                    view_type: vk::ImageViewType::TYPE_2D,
                    initialising_layout: vk::ImageLayout::PREINITIALIZED,
                    expected_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                    layer_count: 1,
                    host_visible: false
                }
            },

            // Typical sky box (cube map)
            (ImageUsage::Skybox, TexturePixelFormat::Rgba) => {
                if init_layer_data.is_none() {
                    return Err(VkError::OpFailed(
                        String::from("Not initialising cube map texture not allowed")));
                }
                ImageCreationParams {
                    format: vk::Format::R8G8B8A8_UNORM,
                    usage: vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED,
                    aspect: vk::ImageAspectFlags::COLOR,
                    view_type: vk::ImageViewType::CUBE,
                    initialising_layout: vk::ImageLayout::PREINITIALIZED,
                    expected_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                    layer_count: 6,
                    host_visible: false
                }
            },

            // Unhandled cases
            _ => {
                return Err(VkError::OpFailed(
                    String::from("Tried to create an image with an unhandled config")));
            }
        };

        let image = Self::make_image(
            context,
            width,
            height,
            &creation_params)?;

        let (allocator, transfer_queue) = context.get_mem_allocator();
        let allocation = allocator.back_image_memory(
            transfer_queue,
            &image,
            creation_params.aspect,
            width,
            height,
            init_layer_data,
            creation_params.initialising_layout,
            creation_params.expected_layout)?;

        let image_view = Self::make_image_view(
            context,
            image,
            &creation_params)?;

        Ok(ImageWrapper {
            allocation,
            image,
            image_view,
            format: creation_params.format
        })
    }

    /// Create the image
    unsafe fn make_image(
        context: &VkContext,
        width: u32,
        height: u32,
        creation_params: &ImageCreationParams
    ) -> Result<vk::Image, VkError> {
        let extent3d = vk::Extent3D { width, height, depth: 1 };
        let flags = match creation_params.view_type {
            vk::ImageViewType::CUBE => vk::ImageCreateFlags::CUBE_COMPATIBLE,
            _ => vk::ImageCreateFlags::empty()
        };
        let image_info = vk::ImageCreateInfo::builder()
            .image_type(vk::ImageType::TYPE_2D)
            .flags(flags)
            .format(creation_params.format)
            .extent(extent3d)
            .mip_levels(1)
            .array_layers(creation_params.layer_count)
            .samples(vk::SampleCountFlags::TYPE_1)
            .tiling(vk::ImageTiling::OPTIMAL)
            .usage(creation_params.usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .initial_layout(creation_params.initialising_layout)
            .build();
        let image = context.device.create_image(&image_info, None)
            .map_err(|e| {
                VkError::OpFailed(format!("Error creating image: {:?}", e))
            })?;

        Ok(image)
    }

    /// Create the image view
    unsafe fn make_image_view(
        context: &VkContext,
        image: vk::Image,
        creation_params: &ImageCreationParams
    ) -> Result<vk::ImageView, VkError> {
        let subresource_range = vk::ImageSubresourceRange::builder()
            .aspect_mask(creation_params.aspect)
            .base_mip_level(0)
            .level_count(1)
            .base_array_layer(0)
            .layer_count(creation_params.layer_count);
        let image_view_create_info = vk::ImageViewCreateInfo::builder()
            .image(image)
            .view_type(creation_params.view_type)
            .format(creation_params.format)
            .subresource_range(*subresource_range);
        let image_view = context.device
            .create_image_view(&image_view_create_info, None)
            .map_err(|e| {
                VkError::OpFailed(format!("{:?}", e))
            })?;

        Ok(image_view)
    }

    /// Destroy all resources held by the instance
    pub unsafe fn destroy(
        &self,
        device: &Device,
        allocator: &MemoryAllocator
    ) -> Result<(), VkError> {
        device.destroy_image_view(self.image_view, None);
        allocator.destroy_image(self.image, &self.allocation)
    }
}
