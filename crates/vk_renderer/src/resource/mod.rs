pub mod buffer;
pub mod image;
pub mod util;

use crate::{
    BufferWrapper, ImageWrapper, RenderpassWrapper, PipelineWrapper, VkError, VkContext,
    OffscreenFramebufferWrapper
};
use resource::{
    ResourceLoader, BufferUsage, VboCreationData, TextureCreationData, ShaderCreationData,
    RenderpassCreationData, RenderpassTarget, PipelineCreationData, ResourceManager,
    PipelineLayoutCreationData, DescriptorSetLayoutCreationData, UboUsage, OffscreenFramebufferData,
    Handle, HandleInterface, Resource
};
use ash::vk;

impl Resource<VkContext> for vk::ShaderModule {
    fn release(&self, loader: &VkContext) {
        unsafe {
            loader.device.destroy_shader_module(*self, None);
        }
    }
}

impl Resource<VkContext> for vk::DescriptorSetLayout {
    fn release(&self, loader: &VkContext) {
        unsafe {
            loader.device.destroy_descriptor_set_layout(*self, None);
        }
    }
}

impl Resource<VkContext> for vk::PipelineLayout {
    fn release(&self, loader: &VkContext) {
        unsafe {
            loader.device.destroy_pipeline_layout(*self, None);
        }
    }
}

impl ResourceLoader for VkContext {

    type VertexBufferHandle = BufferWrapper;
    type TextureHandle = ImageWrapper;
    type ShaderHandle = vk::ShaderModule;
    type OffscreenFramebufferHandle = OffscreenFramebufferWrapper;
    type RenderpassHandle = RenderpassWrapper;
    type DescriptorSetLayoutHandle = vk::DescriptorSetLayout;
    type PipelineLayoutHandle = vk::PipelineLayout;
    type PipelineHandle = PipelineWrapper;
    type LoadError = VkError;

    fn get_current_swapchain_extent(&self) -> Result<(u32, u32), VkError> {
        let extent = self.get_extent()?;
        Ok((extent.width, extent.height))
    }

    fn load_model<T: Sized>(
        &self,
        raw_data: &VboCreationData<T>
    ) -> Result<BufferWrapper, VkError> {
        let buffer = unsafe {
            BufferWrapper::new::<T>(
                self,
                BufferUsage::InitialiseOnceVertexBuffer,
                raw_data.vertex_count * std::mem::size_of::<T>(),
                raw_data.vertex_count,
                Some(&raw_data.vertex_data))?
        };
        Ok(buffer)
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

    fn load_shader(&self, raw_data: &ShaderCreationData) -> Result<vk::ShaderModule, VkError> {
        unsafe {
            let shader_create_info = vk::ShaderModuleCreateInfo::builder()
                .code(raw_data.data);
            self.device
                .create_shader_module(&shader_create_info, None)
                .map_err(|e| VkError::OpFailed(format!("{:?}", e)))
        }
    }

    fn load_offscreen_framebuffer(
        &self,
        raw_data: &OffscreenFramebufferData
    ) -> Result<OffscreenFramebufferWrapper, VkError> {
        let framebuffer = unsafe {
            OffscreenFramebufferWrapper::new(
                self,
                raw_data.width,
                raw_data.height,
                raw_data.color_format,
                raw_data.depth_format)?
        };
        Ok(framebuffer)
    }

    fn load_renderpass(
        &self,
        raw_data: &RenderpassCreationData,
        resource_manager: &ResourceManager<VkContext>
    ) -> Result<RenderpassWrapper, VkError> {
        match raw_data.target {
            RenderpassTarget::SwapchainImageWithDepth => {
                let renderpass = RenderpassWrapper::new_with_swapchain_target(
                    self,
                    raw_data.swapchain_image_index)?;
                Ok(renderpass)
            },
            RenderpassTarget::OffscreenImageWithDepth(framebuffer_index, _, _) => {
                let framebuffer = resource_manager
                    .get_item::<OffscreenFramebufferWrapper>(
                        Handle::from_parts(framebuffer_index, 0))
                    .unwrap();
                let renderpass = RenderpassWrapper::new_with_offscreen_target(
                    self,
                    &framebuffer)?;
                Ok(renderpass)
            }
        }
    }

    fn load_descriptor_set_layout(
        &self,
        raw_data: &DescriptorSetLayoutCreationData
    ) -> Result<vk::DescriptorSetLayout, VkError> {
        let ubo_stage_flags = match raw_data.ubo_usage {
            UboUsage::VertexShaderRead =>
                vk::ShaderStageFlags::VERTEX,
            UboUsage::VertexAndFragmentShaderRead =>
                vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT
        };
        let descriptor_set_layout_binding_infos: Vec<vk::DescriptorSetLayoutBinding> = {
            let mut bindings = vec![vk::DescriptorSetLayoutBinding::builder()
                .binding(0)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .descriptor_count(1)
                .stage_flags(ubo_stage_flags)
                .build()];
            //TODO - for index in 0..texture_image_views.len() { with binding 1 + index
            bindings.push(vk::DescriptorSetLayoutBinding::builder()
                .binding(1)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::FRAGMENT)
                .build());
            bindings
        };
        let descriptor_set_layout_info = vk::DescriptorSetLayoutCreateInfo::builder()
            .bindings(descriptor_set_layout_binding_infos.as_slice());
        let descriptor_set_layout = unsafe {
            self.device
                .create_descriptor_set_layout(&descriptor_set_layout_info, None)
                .map_err(|e|
                    VkError::OpFailed(format!("Error creating descriptor set layout: {:?}", e))
                )?
        };
        Ok(descriptor_set_layout)
    }

    fn load_pipeline_layout(
        &self,
        raw_data: &PipelineLayoutCreationData,
        resource_manager: &ResourceManager<VkContext>
    ) -> Result<vk::PipelineLayout, VkError> {
        let descriptor_set_layout = resource_manager
            .get_item::<vk::DescriptorSetLayout>(
                Handle::from_parts(raw_data.descriptor_set_layout_index, 0))
            .unwrap();
        let pipeline_descriptor_layouts = [*descriptor_set_layout];
        let pipeline_layout_info = vk::PipelineLayoutCreateInfo::builder()
            .set_layouts(&pipeline_descriptor_layouts);
        let pipeline_layout = unsafe {
            self.device
                .create_pipeline_layout(&pipeline_layout_info, None)
                .map_err(|e| VkError::OpFailed(format!("{:?}", e)))?
        };
        Ok(pipeline_layout)
    }

    fn load_pipeline(
        &self,
        raw_data: &PipelineCreationData,
        resource_manager: &ResourceManager<VkContext>,
        swapchain_image_index: usize
    ) -> Result<PipelineWrapper, VkError> {

        let render_extent = self.get_extent()?;
        let mut pipeline = PipelineWrapper::new();
        unsafe {
            pipeline.create_resources(
                self,
                resource_manager,
                swapchain_image_index,
                raw_data.renderpass_index,
                raw_data.descriptor_set_layout_id,
                raw_data.pipeline_layout_index,
                raw_data.vbo_index,
                raw_data.fragment_shader_index,
                raw_data.vbo_index,
                raw_data.vbo_stride_bytes,
                raw_data.ubo_size_bytes,
                false,
                raw_data.texture_index,
                false,
                render_extent
            )?;
        }
        Ok(pipeline)
    }

    #[inline]
    fn make_error(message: String) -> VkError {
        VkError::MissingResource(message)
    }
}
