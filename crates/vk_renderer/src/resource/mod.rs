pub mod buffer;
pub mod image;
pub mod util;

use crate::{VkError, VkContext};
use ecs::{EcsManager, Handle, resource::{Resource, ResourceLoader}};
use ash::vk;

/// ShaderStage enum
/// Used to signal what point in the pipeline a shader should be used
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum ShaderStage {
    Vertex,
    Fragment
}

/// ShaderCreationData struct
/// Information needed to prepare a reusable shader ahead of time
pub struct ShaderCreationData {
    pub data: &'static [u32],
    pub stage: ShaderStage
}

/// UboUsage enum
/// Used to signal how a UBO is going to be used
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum UboUsage {
    VertexShaderRead,
    VertexAndFragmentShaderRead
}

/// DescriptorSetLayoutCreationData struct
/// Information needed to describe a descriptor set layout
pub struct DescriptorSetLayoutCreationData {
    pub ubo_usage: UboUsage
}

/// PipelineLayoutCreationData struct
/// Information needed to describe a pipeline layout
pub struct PipelineLayoutCreationData {
    pub descriptor_set_layout_index: u32
}

impl Resource<VkContext, > for vk::ShaderModule {
    type CreationData = ShaderCreationData;

    fn create(
        loader: &VkContext,
        _ecs: &EcsManager<VkContext>,
        data: &ShaderCreationData
    ) -> Result<Self, VkError> {
        unsafe {
            let shader_create_info = vk::ShaderModuleCreateInfo::builder()
                .code(data.data);
            loader.device
                .create_shader_module(&shader_create_info, None)
                .map_err(|e| VkError::OpFailed(format!("{:?}", e)))
        }
    }

    fn release(&self, loader: &VkContext) {
        unsafe {
            loader.device.destroy_shader_module(*self, None);
        }
    }
}

impl Resource<VkContext> for vk::DescriptorSetLayout {
    type CreationData = DescriptorSetLayoutCreationData;

    fn create(
        loader: &VkContext,
        _ecs: &EcsManager<VkContext>,
        data: &DescriptorSetLayoutCreationData
    ) -> Result<Self, VkError> {
        let ubo_stage_flags = match data.ubo_usage {
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
            loader.device
                .create_descriptor_set_layout(&descriptor_set_layout_info, None)
                .map_err(|e|
                    VkError::OpFailed(format!("Error creating descriptor set layout: {:?}", e))
                )?
        };
        Ok(descriptor_set_layout)
    }

    fn release(&self, loader: &VkContext) {
        unsafe {
            loader.device.destroy_descriptor_set_layout(*self, None);
        }
    }
}

impl Resource<VkContext> for vk::PipelineLayout {
    type CreationData = PipelineLayoutCreationData;

    fn create(
        loader: &VkContext,
        ecs: &EcsManager<VkContext>,
        data: &PipelineLayoutCreationData
    ) -> Result<Self, VkError> {
        let descriptor_set_layout  = ecs
            .get_item::<vk::DescriptorSetLayout>(
                Handle::for_resource(data.descriptor_set_layout_index))
            .unwrap();
        let pipeline_descriptor_layouts = [*descriptor_set_layout];
        let pipeline_layout_info = vk::PipelineLayoutCreateInfo::builder()
            .set_layouts(&pipeline_descriptor_layouts);
        let pipeline_layout = unsafe {
            loader.device
                .create_pipeline_layout(&pipeline_layout_info, None)
                .map_err(|e| VkError::OpFailed(format!("{:?}", e)))?
        };
        Ok(pipeline_layout)
    }

    fn release(&self, loader: &VkContext) {
        unsafe {
            loader.device.destroy_pipeline_layout(*self, None);
        }
    }
}

impl ResourceLoader for VkContext {
    type LoadError = VkError;

    fn get_current_swapchain_extent(&self) -> Result<(u32, u32), VkError> {
        let extent = self.get_extent()?;
        Ok((extent.width, extent.height))
    }

    #[inline]
    fn make_error(message: String) -> VkError {
        VkError::MissingResource(message)
    }
}
