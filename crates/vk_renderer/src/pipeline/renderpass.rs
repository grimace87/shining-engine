
use crate::{VkContext, VkError, OffscreenFramebufferWrapper, TexturePixelFormat};
use ecs::{EcsManager, Handle, resource::Resource};
use ash::vk;

/// RenderpassTarget enum
/// Used to signal what arrangement of attachments and subpasses will be used in a renderpass
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum RenderpassTarget {

    // Will require one renderpass per swapchain image
    SwapchainImageWithDepth,

    // Contains the index of the offscreen framebuffer, then the width, then the height
    OffscreenImageWithDepth(u32, u32, u32)
}

/// RenderpassCreationData struct
/// Information needed to prepare a (potentially reusable) renderpass ahead of time
pub struct RenderpassCreationData {
    pub target: RenderpassTarget,
    pub swapchain_image_index: usize
}

/// RenderpassWrapper struct
/// Wraps resources related to renderpasses, including framebuffers. Resources need to be recreated
/// if the swapchain is recreated.
pub struct RenderpassWrapper {
    pub renderpass: vk::RenderPass,
    pub swapchain_framebuffer: vk::Framebuffer,
    pub custom_framebuffer: Option<vk::Framebuffer>
}

impl Resource<VkContext> for RenderpassWrapper {
    type CreationData = RenderpassCreationData;

    fn create(
        loader: &VkContext,
        ecs: &EcsManager<VkContext>,
        data: &RenderpassCreationData
    ) -> Result<Self, VkError> {
        match data.target {
            RenderpassTarget::SwapchainImageWithDepth => {
                let renderpass = RenderpassWrapper::new_with_swapchain_target(
                    loader,
                    data.swapchain_image_index)?;
                Ok(renderpass)
            },
            RenderpassTarget::OffscreenImageWithDepth(framebuffer_index, _, _) => {
                let framebuffer  = ecs
                    .get_item::<OffscreenFramebufferWrapper>(
                        Handle::for_resource(framebuffer_index))
                    .unwrap();
                let renderpass = RenderpassWrapper::new_with_offscreen_target(
                    loader,
                    &framebuffer)?;
                Ok(renderpass)
            }
        }
    }

    fn release(&self, loader: &VkContext) {
        unsafe {
            loader.device.destroy_framebuffer(self.swapchain_framebuffer, None);
            if let Some(framebuffer) = self.custom_framebuffer.as_ref() {
                loader.device.destroy_framebuffer(*framebuffer, None);
            }
            loader.device.destroy_render_pass(self.renderpass, None);
        }
    }
}

impl RenderpassWrapper {

    /// Create a new instance for rendering to a swapchain image, with all resources initialised
    pub fn new_with_swapchain_target(
        context: &VkContext,
        image_index: usize
    ) -> Result<RenderpassWrapper, VkError> {
        let mut wrapper = RenderpassWrapper {
            renderpass: vk::RenderPass::null(),
            swapchain_framebuffer: vk::Framebuffer::null(),
            custom_framebuffer: None
        };
        unsafe {
            wrapper.create_swapchain_renderpass_resources(
                context,
                image_index)?;
        }
        Ok(wrapper)
    }

    /// Create a new instance, with all resources initialised
    pub fn new_with_offscreen_target(
        context: &VkContext,
        target: &OffscreenFramebufferWrapper
    ) -> Result<RenderpassWrapper, VkError> {
        let mut wrapper = RenderpassWrapper {
            renderpass: vk::RenderPass::null(),
            swapchain_framebuffer: vk::Framebuffer::null(),
            custom_framebuffer: None
        };
        unsafe {
            wrapper.create_offscreen_renderpass_resources(
                context,
                target,
                true)?;
        }
        Ok(wrapper)
    }

    /// Create all resources for rendering into a swapchain image
    unsafe fn create_swapchain_renderpass_resources(
        &mut self,
        context: &VkContext,
        image_index: usize
    ) -> Result<(), VkError> {

        let depth_image = match context.get_depth_image() {
            Some(image) => image,
            _ => return Err(VkError::OpFailed(
                String::from("Creating new renderpass wrapper with no depth image available")
            ))
        };

        // Define subpass with single colour attachment
        let surface_format = context.get_surface_format().format;
        let attachments = [
            vk::AttachmentDescription::builder()
                .format(surface_format)
                .load_op(vk::AttachmentLoadOp::CLEAR)
                .store_op(vk::AttachmentStoreOp::STORE)
                .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
                .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
                .initial_layout(vk::ImageLayout::UNDEFINED)
                .final_layout(vk::ImageLayout::PRESENT_SRC_KHR)
                .samples(vk::SampleCountFlags::TYPE_1)
                .build(),
            vk::AttachmentDescription::builder()
                .format(depth_image.format)
                .load_op(vk::AttachmentLoadOp::CLEAR)
                .store_op(vk::AttachmentStoreOp::DONT_CARE)
                .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
                .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
                .initial_layout(vk::ImageLayout::UNDEFINED)
                .final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
                .samples(vk::SampleCountFlags::TYPE_1)
                .build()
        ];
        let color_attachment_refs = [
            vk::AttachmentReference {
                attachment: 0,
                layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL
            }
        ];
        let depth_attachment_ref = vk::AttachmentReference {
            attachment: 1,
            layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL
        };
        let subpasses = [
            vk::SubpassDescription::builder()
                .color_attachments(&color_attachment_refs)
                .depth_stencil_attachment(&depth_attachment_ref)
                .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
                .build()
        ];
        let subpass_dependencies = [
            vk::SubpassDependency::builder()
                .src_subpass(vk::SUBPASS_EXTERNAL)
                .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
                .dst_subpass(0)
                .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
                .dst_access_mask(
                    vk::AccessFlags::COLOR_ATTACHMENT_READ | vk::AccessFlags::COLOR_ATTACHMENT_WRITE
                )
                .build()
        ];

        // Create the renderpass with this one subpass
        let renderpass_info = vk::RenderPassCreateInfo::builder()
            .attachments(&attachments)
            .subpasses(&subpasses)
            .dependencies(&subpass_dependencies);
        let renderpass = context.device
            .create_render_pass(&renderpass_info, None)
            .map_err(|e| {
                VkError::OpFailed(format!("{:?}", e))
            })?;

        // Create framebuffers for the swapchain image views for use in this renderpass
        let framebuffer = self.create_swapchain_framebuffer(
            context,
            image_index,
            renderpass)?;

        self.renderpass = renderpass;
        self.swapchain_framebuffer = framebuffer;
        self.custom_framebuffer = None;

        Ok(())
    }

    /// Create all resources for rendering into an offscreen framebuffer
    unsafe fn create_offscreen_renderpass_resources(
        &mut self,
        context: &VkContext,
        target: &OffscreenFramebufferWrapper,
        discard_existing_image_content: bool
    ) -> Result<(), VkError> {

        // TODO - Something useful with this flag
        if !discard_existing_image_content {
            panic!(
                "Unhandled case RenderpassWrapper::create_offscreen_renderpass_resources with \
                discard_existing_image_content set to false"
            );
        }

        // Get the texture to use for color attachment
        let color_format = match target.color_format {
            TexturePixelFormat::Rgba => vk::Format::R8G8B8A8_UNORM,
            _ => return Err(VkError::OpFailed(
                format!("Cannot set color attachment to {:?}", target.color_format)))
        };

        // Define subpass with single colour attachment and optionally depth attachment
        let initial_layout = vk::ImageLayout::UNDEFINED;
        let mut attachments = vec![vk::AttachmentDescription::builder()
            .format(color_format)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(initial_layout)
            .final_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .samples(vk::SampleCountFlags::TYPE_1)
            .build()];
        let depth_texture_image_view = match &target.depth_texture {
            Some(depth_texture) => {
                // Get the texture to use for depth attachment
                match target.depth_format {
                    TexturePixelFormat::Unorm16 => {
                        attachments.push(vk::AttachmentDescription::builder()
                            .format(vk::Format::D16_UNORM)
                            .load_op(vk::AttachmentLoadOp::CLEAR)
                            .store_op(vk::AttachmentStoreOp::DONT_CARE)
                            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
                            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
                            .initial_layout(initial_layout)
                            .final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
                            .samples(vk::SampleCountFlags::TYPE_1)
                            .build());
                    },
                    _ => return Err(VkError::OpFailed(
                        format!("Cannot set depth attachment tp {:?}", target.depth_format))
                    )
                };
                Some(depth_texture.image_view)
            },
            _ => None
        };

        let color_attachment_refs = [
            vk::AttachmentReference {
                attachment: 0,
                layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL
            }
        ];

        let depth_attachment_ref = vk::AttachmentReference {
            attachment: 1,
            layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL
        };
        let subpasses = {
            let subpass_description = vk::SubpassDescription::builder()
                .color_attachments(&color_attachment_refs)
                .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS);
            if target.depth_texture.is_some() {
                [subpass_description.depth_stencil_attachment(&depth_attachment_ref).build()]
            } else {
                [subpass_description.build()]
            }
        };

        let subpass_dependencies = [
            vk::SubpassDependency::builder()
                .src_subpass(vk::SUBPASS_EXTERNAL)
                .src_stage_mask(vk::PipelineStageFlags::FRAGMENT_SHADER)
                .src_access_mask(vk::AccessFlags::SHADER_READ)
                .dst_subpass(0)
                .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
                .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE)
                .build(),
            vk::SubpassDependency::builder()
                .src_subpass(0)
                .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
                .src_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE)
                .dst_subpass(vk::SUBPASS_EXTERNAL)
                .dst_stage_mask(vk::PipelineStageFlags::FRAGMENT_SHADER)
                .dst_access_mask(vk::AccessFlags::SHADER_READ)
                .build()
        ];

        // Create the renderpass with this one subpass
        let renderpass_info = vk::RenderPassCreateInfo::builder()
            .attachments(attachments.as_slice())
            .subpasses(&subpasses)
            .dependencies(&subpass_dependencies);
        let renderpass = context.device
            .create_render_pass(&renderpass_info, None)
            .map_err(|e| {
                VkError::OpFailed(format!("{:?}", e))
            })?;

        // Create framebuffers for swapchain image views, or new framebuffers from scratch, for use in this renderpass
        self.renderpass = renderpass;
        self.swapchain_framebuffer = vk::Framebuffer::null();
        self.custom_framebuffer = Some(Self::create_offscreen_framebuffer(
            context,
            renderpass,
            target,
            target.color_texture.image_view,
            depth_texture_image_view)?);

        Ok(())
    }

    /// Create a framebuffer for rendering into a swapchain image
    unsafe fn create_swapchain_framebuffer(
        &self,
        context: &VkContext,
        image_index: usize,
        renderpass: vk::RenderPass
    ) -> Result<vk::Framebuffer, VkError> {
        let extent = context.get_extent()?;
        let image_view = context.get_swapchain_image_view(image_index)?;
        let depth_image = context.get_depth_image().unwrap();
        let attachments_array = [
            image_view,
            depth_image.image_view
        ];
        let framebuffer_info = vk::FramebufferCreateInfo::builder()
            .render_pass(renderpass)
            .attachments(&attachments_array)
            .width(extent.width)
            .height(extent.height)
            .layers(1);
        let framebuffer = context.device
            .create_framebuffer(&framebuffer_info, None)
            .map_err(|e| {
                VkError::OpFailed(format!("{:?}", e))
            })?;
        Ok(framebuffer)
    }

    /// Create a framebuffer for rendering into an offscreen image
    unsafe fn create_offscreen_framebuffer(
        context: &VkContext,
        renderpass: vk::RenderPass,
        target: &OffscreenFramebufferWrapper,
        color_image: vk::ImageView,
        depth_image: Option<vk::ImageView>
    ) -> Result<vk::Framebuffer, VkError> {

        let width = target.width as u32;
        let height = target.height as u32;

        let mut attachment_image_view = vec![color_image];
        if let Some(image_view) = depth_image.as_ref() {
            attachment_image_view.push(*image_view);
        }

        let framebuffer_info = vk::FramebufferCreateInfo::builder()
            .render_pass(renderpass)
            .attachments(attachment_image_view.as_slice())
            .width(width)
            .height(height)
            .layers(1);
        context.device
            .create_framebuffer(&framebuffer_info, None)
            .map_err(|e| {
                VkError::OpFailed(format!("{:?}", e))
            })
    }
}
