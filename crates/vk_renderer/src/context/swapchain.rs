
use crate::{VkCore, VkContext, ImageWrapper, ImageUsage, TexturePixelFormat};
use ecs::resource::Resource;
use error::EngineError;
use ash::{
    vk,
    Device,
    extensions::khr::{
        Surface,
        Swapchain
    }
};
use std::cmp::max;

pub const MIN_SWAPCHAIN_SIZE: u32 = 2;
pub const MAX_SWAPCHAIN_SIZE: u32 = 3;

pub struct SwapchainWrapper {
    swapchain: vk::SwapchainKHR,
    surface_format: vk::SurfaceFormatKHR,
    image_views: Vec<vk::ImageView>,
    depth_image: Option<ImageWrapper>
}

impl Default for SwapchainWrapper {

    fn default() -> Self {
        SwapchainWrapper {
            swapchain: vk::SwapchainKHR::null(),
            surface_format: vk::SurfaceFormatKHR::default(),
            image_views: vec![],
            depth_image: None
        }
    }
}

impl SwapchainWrapper {

    pub unsafe fn new(
        core: &VkCore,
        context: &VkContext,
        surface_fn: &Surface,
        surface: vk::SurfaceKHR,
        extent: vk::Extent2D
    ) -> Result<SwapchainWrapper, EngineError> {
        let (swapchain, surface_format) = Self::create_swapchain(
            core,
            surface_fn,
            surface,
            &context.swapchain_fn,
            vk::SwapchainKHR::null())?;
        let image_views =
            Self::create_swapchain_image_views(
                &context.device,
                &context.swapchain_fn,
                swapchain)?;
        let depth_image = ImageWrapper::new(
            context,
            ImageUsage::DepthBuffer,
            TexturePixelFormat::Unorm16,
            extent.width as u32,
            extent.height as u32,
            None)?;

        Ok(SwapchainWrapper {
            swapchain,
            surface_format,
            image_views,
            depth_image: Some(depth_image)
        })
    }

    pub unsafe fn destroy(&self, context: &VkContext, swapchain_fn: &Swapchain) {
        if let Some(image) = &self.depth_image {
            image.release(context);
        }
        for image_view in self.image_views.iter() {
            context.device.destroy_image_view(*image_view, None);
        }
        swapchain_fn.destroy_swapchain(self.swapchain, None);
    }

    pub fn get_surface_format(&self) -> vk::SurfaceFormatKHR {
        self.surface_format
    }

    pub fn get_image_count(&self) -> usize {
        self.image_views.len()
    }

    pub fn get_image_view(&self, index: usize) -> Result<vk::ImageView, EngineError> {
        if index >= self.image_views.len() {
            return Err(EngineError::EngineError(format!("Bad swapchain index: {}", index)));
        }
        Ok(self.image_views[index])
    }

    pub fn get_depth_image(&self) -> Option<&ImageWrapper> {
        match &self.depth_image {
            Some(image) => Some(image),
            _ => None
        }
    }

    pub fn get_swapchain(&self) -> vk::SwapchainKHR {
        self.swapchain
    }

    /// Create a swapchain; ensures that it is supported by the device and surface
    unsafe fn create_swapchain(
        core: &VkCore,
        surface_fn: &Surface,
        surface: vk::SurfaceKHR,
        swapchain_fn: &Swapchain,
        previous_swapchain: vk::SwapchainKHR
    ) -> Result<(vk::SwapchainKHR, vk::SurfaceFormatKHR), EngineError> {

        // Check for support and get some known-supported parameters
        let (
            min_image_count,
            current_extent,
            current_transform
        ) = Self::validate_basic_requirements(
            core,
            surface_fn,
            surface)?;
        let present_mode = Self::choose_present_mode(core.physical_device, surface_fn, surface)?;
        let surface_format = Self::choose_surface_format(core.physical_device, surface_fn, surface)?;

        // Create the swapchain
        let swapchain_create_info = vk::SwapchainCreateInfoKHR::builder()
            .surface(surface)
            .min_image_count(min_image_count)
            .image_color_space(surface_format.color_space)
            .image_format(surface_format.format)
            .image_extent(current_extent)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            .pre_transform(current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .clipped(true)
            .image_array_layers(1)
            .old_swapchain(previous_swapchain);
        let swapchain = swapchain_fn.create_swapchain(&swapchain_create_info, None)
            .map_err(|e| {
                EngineError::OpFailed(format!("{:?}", e))
            })?;

        Ok((swapchain, surface_format))
    }

    /// Create the image views for the swapchain
    unsafe fn create_swapchain_image_views(
        device: &Device,
        swapchain_fn: &Swapchain,
        swapchain: vk::SwapchainKHR
    ) -> Result<Vec<vk::ImageView>, EngineError> {
        // Make the image views over the images
        let swapchain_images = swapchain_fn.get_swapchain_images(swapchain)
            .map_err(|e| {
                EngineError::OpFailed(format!("{:?}", e))
            })?;
        let image_views: Vec<_> = swapchain_images.iter()
            .map(|image| {
                let subresource_range = vk::ImageSubresourceRange::builder()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .base_mip_level(0)
                    .level_count(1)
                    .base_array_layer(0)
                    .layer_count(1);
                let image_view_create_info = vk::ImageViewCreateInfo::builder()
                    .image(*image)
                    .view_type(vk::ImageViewType::TYPE_2D)
                    .format(vk::Format::B8G8R8A8_UNORM)
                    .subresource_range(*subresource_range);
                device.create_image_view(&image_view_create_info, None)
                    .map_err(|e| {
                        format!("Error creating image views for swapchain: {:?}", e);
                        EngineError::OpFailed(format!("{:?}", e))
                    })
                    .unwrap()
            })
            .collect();
        Ok(image_views)
    }

    /// Validates that the physical device and surface supported everything needed
    unsafe fn validate_basic_requirements(
        core: &VkCore,
        surface_fn: &Surface,
        surface: vk::SurfaceKHR
    ) -> Result<(u32, vk::Extent2D, vk::SurfaceTransformFlagsKHR), EngineError> {
        let physical_device = core.physical_device;
        let graphics_queue_family_index = core.graphics_queue_family_index;

        let present_supported = surface_fn
            .get_physical_device_surface_support(physical_device, graphics_queue_family_index, surface)
            .map_err(|e| {
                EngineError::OpFailed(format!("{:?}", e))
            })?;
        if !present_supported {
            return Err(EngineError::OpFailed(
                String::from("Presentation not supported by selected graphics queue family")));
        }

        let surface_capabilities = surface_fn
            .get_physical_device_surface_capabilities(physical_device, surface)
            .map_err(|e| {
                EngineError::OpFailed(format!("{:?}", e))
            })?;

        let max_too_small = surface_capabilities.max_image_count != 0 &&
            surface_capabilities.max_image_count < MIN_SWAPCHAIN_SIZE;
        let min_too_large = surface_capabilities.min_image_count > MAX_SWAPCHAIN_SIZE;
        if max_too_small || min_too_large {
            return Err(EngineError::OpFailed(
                String::from("Requested swapchain size is not supported")));
        }

        let images_to_request =
            max(MIN_SWAPCHAIN_SIZE, surface_capabilities.min_image_count);
        Ok((
            images_to_request,
            surface_capabilities.current_extent,
            surface_capabilities.current_transform
        ))
    }

    /// Select a present mode, ensuring it is supported (FIFO is considered the preferred option)
    unsafe fn choose_present_mode(
        physical_device: vk::PhysicalDevice,
        surface_fn: &Surface,
        surface: vk::SurfaceKHR
    ) -> Result<vk::PresentModeKHR, EngineError> {
        let surface_present_modes = surface_fn
            .get_physical_device_surface_present_modes(physical_device, surface)
            .map_err(|e| {
                EngineError::OpFailed(format!("{:?}", e))
            })?;
        if !surface_present_modes.contains(&vk::PresentModeKHR::FIFO) {
            return Err(EngineError::OpFailed(
                String::from(
                    "FIFO presentation mode not supported by selected graphics queue family")));
        }
        Ok(vk::PresentModeKHR::FIFO)
    }

    /// Select a supported surface format
    unsafe fn choose_surface_format(
        physical_device: vk::PhysicalDevice,
        surface_fn: &Surface,
        surface: vk::SurfaceKHR
    ) -> Result<vk::SurfaceFormatKHR, EngineError> {
        let surface_formats = surface_fn
            .get_physical_device_surface_formats(physical_device, surface)
            .map_err(|e| {
                EngineError::OpFailed(format!("{:?}", e))
            })?;
        if surface_formats.is_empty() {
            return Err(EngineError::OpFailed(
                String::from("No surface formats supported")));
        }
        let index_of_desired = surface_formats.iter().position(|f| {
            f.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR && f.format == vk::Format::B8G8R8A8_UNORM
        });
        let format: vk::SurfaceFormatKHR = match index_of_desired {
            Some(i) => surface_formats[i],
            None => *surface_formats.first().unwrap()
        };
        Ok(format)
    }
}
