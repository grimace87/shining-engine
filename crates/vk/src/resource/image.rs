
use crate::{
    VkError,
    context::VkContext,
    mem::{MemoryAllocator, MemoryAllocation, ManagesImageMemory, ManagesMemoryTransfers}
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
    layer_count: u32,
    pre_initialised: bool
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
                    layer_count: 1,
                    pre_initialised: false
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
                    layer_count: 1,
                    pre_initialised: false
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
                    layer_count: 1,
                    pre_initialised: false
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
                    layer_count: 1,
                    pre_initialised: true
                }
            },

            // Typical sky box (cube map)
            (ImageUsage::Skybox, TexturePixelFormat::Rgba) => {
                if init_layer_data.is_none() {
                    return Err(VkError::OpFailed(
                        String::from("Not initialising sample-only texture not allowed")));
                }
                ImageCreationParams {
                    format: vk::Format::R8G8B8A8_UNORM,
                    usage: vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED,
                    aspect: vk::ImageAspectFlags::COLOR,
                    view_type: vk::ImageViewType::CUBE,
                    layer_count: 6,
                    pre_initialised: true
                }
            },

            // Unhandled cases
            _ => {
                return Err(VkError::OpFailed(
                    String::from("Tried to create an image with an unhandled config")));
            }
        };

        let (allocation, image, image_view) = Self::make_image_and_view(
            context,
            width,
            height,
            &creation_params)?;

        if let Some(layer_data) = init_layer_data {
            Self::initialise_read_only_color_texture(
                context,
                width,
                height,
                &image,
                &allocation,
                layer_data)?;
        }

        Ok(ImageWrapper {
            allocation,
            image,
            image_view,
            format: creation_params.format
        })
    }

    /// Create the image and image view
    unsafe fn make_image_and_view(
        context: &VkContext,
        width: u32,
        height: u32,
        creation_params: &ImageCreationParams
    ) -> Result<(MemoryAllocation, vk::Image, vk::ImageView), VkError> {
        let queue_families = [
            context.graphics_queue.queue_family_index
        ];
        let extent3d = vk::Extent3D { width, height, depth: 1 };
        let flags = match creation_params.view_type {
            vk::ImageViewType::CUBE => vk::ImageCreateFlags::CUBE_COMPATIBLE,
            _ => vk::ImageCreateFlags::empty()
        };

        let initial_layout: vk::ImageLayout = match creation_params.pre_initialised {
            true => vk::ImageLayout::PREINITIALIZED,
            false => vk::ImageLayout::UNDEFINED
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
            .queue_family_indices(&queue_families)
            .initial_layout(initial_layout)
            .build();
        let (allocator, _) = context.get_mem_allocator();
        let (image, allocation) = allocator
            .create_image(&image_info)
            .map_err(|e| {
                VkError::OpFailed(format! ("Allocation error: {:?}", e))
            })?;

        context.device.bind_image_memory(image, allocation.get_memory(), 0)
            .map_err(|e| {
                VkError::OpFailed(format! ("Error binding memory to image: {:?}", e))
            })?;

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

        Ok((allocation, image, image_view))
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

    /// Initialise the image's memory with texture data; may use a staging buffer to allocate
    /// device-local memory and transitions the image into the optimal layout for reading in
    /// samplers in shaders
    unsafe fn initialise_read_only_color_texture(
        context: &VkContext,
        width: u32,
        height: u32,
        image: &vk::Image,
        allocation: &MemoryAllocation,
        layer_data: &[Vec<u8>]) -> Result<(), VkError> {
        if layer_data.is_empty() {
            panic!("Passed empty layer data as ImageWrapper init data")
        }
        let (allocator, transfer_queue) = context.get_mem_allocator();
        allocator.transfer_data_to_new_texture(
            transfer_queue, width, height, image, allocation, layer_data)?;
        Ok(())
    }
}
