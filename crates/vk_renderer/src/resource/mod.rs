pub mod buffer;
pub mod image;
pub mod util;

use crate::{
    BufferWrapper, ImageWrapper, RenderpassWrapper, PipelineWrapper, VkError, VkContext,
    OffscreenFramebufferWrapper
};
use model::StaticVertex;
use resource::{
    ResourceLoader, BufferUsage, VboCreationData, TextureCreationData, ShaderCreationData,
    RenderpassCreationData, RenderpassTarget, PipelineCreationData, ResourceManager,
    PipelineLayoutCreationData, DescriptorSetLayoutCreationData, UboUsage, OffscreenFramebufferData
};
use ash::vk;

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

    fn load_model<T: Sized>(
        &self,
        raw_data: &VboCreationData<T>
    ) -> Result<(BufferWrapper, usize), VkError> {
        let buffer = unsafe {
            BufferWrapper::new::<T>(
                self,
                BufferUsage::InitialiseOnceVertexBuffer,
                raw_data.vertex_count * std::mem::size_of::<T>(),
                raw_data.vertex_count,
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

    fn load_shader(&self, raw_data: &ShaderCreationData) -> Result<vk::ShaderModule, VkError> {
        unsafe {
            let shader_create_info = vk::ShaderModuleCreateInfo::builder()
                .code(raw_data.data);
            self.device
                .create_shader_module(&shader_create_info, None)
                .map_err(|e| VkError::OpFailed(format!("{:?}", e)))
        }
    }

    fn release_shader(&mut self, shader: &vk::ShaderModule) -> Result<(), VkError> {
        unsafe {
            self.device.destroy_shader_module(*shader, None);
        }
        Ok(())
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

    fn release_offscreen_framebuffer(
        &mut self,
        framebuffer: &OffscreenFramebufferWrapper
    ) -> Result<(), VkError> {
        unsafe {
            framebuffer.destroy(self)?;
        }
        Ok(())
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
                    .get_offscreen_framebuffer_handle(framebuffer_index)?;
                let renderpass = RenderpassWrapper::new_with_offscreen_target(
                    self,
                    &framebuffer)?;
                Ok(renderpass)
            }
        }
    }

    fn release_renderpass(
        &mut self,
        renderpass: &RenderpassWrapper
    ) -> Result<(), VkError> {
        renderpass.destroy_resources(self);
        Ok(())
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

    fn release_descriptor_set_layout(
        &mut self,
        descriptor_set_layout: &vk::DescriptorSetLayout
    ) -> Result<(), VkError> {
        unsafe {
            self.device
                .destroy_descriptor_set_layout(*descriptor_set_layout, None);
        }
        Ok(())
    }

    fn load_pipeline_layout(
        &self,
        raw_data: &PipelineLayoutCreationData,
        resource_manager: &ResourceManager<VkContext>
    ) -> Result<vk::PipelineLayout, VkError> {
        let descriptor_set_layout = resource_manager
            .get_descriptor_set_layout_handle(raw_data.descriptor_set_layout_index)?;
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

    fn release_pipeline_layout(
        &mut self,
        pipeline_layout: &vk::PipelineLayout
    ) -> Result<(), VkError> {
        unsafe {
            self.device.destroy_pipeline_layout(*pipeline_layout, None);
        }
        Ok(())
    }

    fn load_pipeline(
        &self,
        raw_data: &PipelineCreationData,
        resource_manager: &ResourceManager<VkContext>,
        current_swapchain_width: u32,
        current_swapchain_height: u32
    ) -> Result<PipelineWrapper, VkError> {

        let renderpass_spec = RenderpassCreationData {
            target: RenderpassTarget::SwapchainImageWithDepth,
            swapchain_image_index: raw_data.swapchain_image_index
        };
        let complex_renderpass_id = renderpass_spec.encode_complex_renderpass_id(
            raw_data.renderpass_index,
            current_swapchain_width,
            current_swapchain_height
        );

        let render_extent = self.get_extent()?;
        let mut pipeline = PipelineWrapper::new();
        unsafe {
            pipeline.create_resources(
                self,
                resource_manager,
                complex_renderpass_id,
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

    fn release_pipeline(
        &mut self,
        pipeline: &PipelineWrapper
    ) -> Result<(), VkError> {
        pipeline.destroy_resources(self);
        Ok(())
    }

    #[inline]
    fn make_error(message: String) -> VkError {
        VkError::MissingResource(message)
    }
}
