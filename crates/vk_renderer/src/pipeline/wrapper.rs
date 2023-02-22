
use crate::{
    VkContext, VkError, BufferWrapper, RenderpassWrapper, ImageWrapper, BufferUsage,
    VboCreationData
};
use resource::{ResourceManager, Resource, Handle};
use ash::vk;
use std::ffi::CString;

/// PipelineCreationData struct
/// Information needed to prepare a (potentially reusable) pipeline ahead of time
pub struct PipelineCreationData {
    pub pipeline_layout_index: u32,
    pub renderpass_index: u32,
    pub descriptor_set_layout_id: u32,
    pub vertex_shader_index: u32,
    pub fragment_shader_index: u32,
    pub vbo_index: u32,
    pub texture_index: u32,
    pub vbo_stride_bytes: u32,
    pub ubo_size_bytes: usize,
    pub swapchain_image_index: usize
}

/// PipelineWrapper struct
/// Resources for a Vulkan pipeline to render a single step within a renderpass within the full
/// rendering description for a particular scene.
pub struct PipelineWrapper {
    vertex_buffer: vk::Buffer,
    vertex_count: usize,
    uniform_buffer: BufferWrapper,
    texture_image_view: vk::ImageView, // TODO - Vec
    sampler: vk::Sampler, // TODO - Vec
    descriptor_pool: vk::DescriptorPool,
    descriptor_set: vk::DescriptorSet,
    pipeline: vk::Pipeline
}

impl Resource<VkContext> for PipelineWrapper {
    type CreationData = PipelineCreationData;

    fn create(
        loader: &VkContext,
        resource_manager: &ResourceManager<VkContext>,
        data: &PipelineCreationData
    ) -> Result<Self, VkError> {

        let render_extent = loader.get_extent()?;
        let mut pipeline = PipelineWrapper::new();
        unsafe {
            pipeline.create_resources(
                loader,
                resource_manager,
                data.swapchain_image_index,
                data.renderpass_index,
                data.descriptor_set_layout_id,
                data.pipeline_layout_index,
                data.vbo_index,
                data.fragment_shader_index,
                data.vbo_index,
                data.vbo_stride_bytes,
                data.ubo_size_bytes,
                false,
                data.texture_index,
                false,
                render_extent
            )?;
        }
        Ok(pipeline)
    }

    fn release(&self, loader: &VkContext) {
        unsafe {
            loader.device.destroy_pipeline(self.pipeline, None);
            self.uniform_buffer.release(loader);
            loader.device.destroy_descriptor_pool(self.descriptor_pool, None);
            loader.device.destroy_sampler(self.sampler, None);
        }
    }
}

impl PipelineWrapper {

    /// Create a new instance with empty fields; requires a separate initialisation call.
    pub fn new() -> PipelineWrapper {
        PipelineWrapper {
            vertex_buffer: vk::Buffer::null(),
            vertex_count: 0,
            uniform_buffer: BufferWrapper::empty(),
            texture_image_view: vk::ImageView::null(),
            sampler: vk::Sampler::null(),
            descriptor_pool: vk::DescriptorPool::null(),
            descriptor_set: vk::DescriptorSet::null(),
            pipeline: vk::Pipeline::null()
        }
    }

    pub fn get_pipeline(&self) -> vk::Pipeline {
        self.pipeline
    }

    pub fn get_descriptor_set(&self) -> vk::DescriptorSet {
        self.descriptor_set
    }

    /// Create resources needed to render a single step within a pass
    pub unsafe fn create_resources(
        &mut self,
        context: &VkContext,
        resource_manager: &ResourceManager<VkContext>,
        swapchain_image_index: usize,
        renderpass_id: u32,
        descriptor_set_layout_id: u32,
        pipeline_layout_index: u32,
        vertex_shader_index: u32,
        fragment_shader_index: u32,
        vbo_index: u32,
        vbo_stride_bytes: u32,
        ubo_size_bytes: usize,
        draw_indexed: bool,
        texture_index: u32,
        depth_test: bool,
        render_extent: vk::Extent2D
    ) -> Result<(), VkError> {

        // Query renderpass and pipeline layout
        let renderpass_wrapper = resource_manager
            .get_item::<RenderpassWrapper>(
                Handle::with_minor_variation(renderpass_id, swapchain_image_index as u32)
                    .unwrap())
            .unwrap();
        let descriptor_set_layout = resource_manager
            .get_item::<vk::DescriptorSetLayout>(
                Handle::with_unique_id(descriptor_set_layout_id, 0))
            .unwrap();
        let pipeline_layout = resource_manager
            .get_item::<vk::PipelineLayout>(
                Handle::with_unique_id(pipeline_layout_index, 0))
            .unwrap();

        // Query shader modules
        let vertex_shader_module = resource_manager
            .get_item::<vk::ShaderModule>(
                Handle::with_unique_id(vertex_shader_index as u32, 0))
            .unwrap();
        let fragment_shader_module = resource_manager
            .get_item::<vk::ShaderModule>(
                Handle::with_unique_id(fragment_shader_index as u32, 0))
            .unwrap();

        // Make shader modules
        let main_function_name = CString::new("main").unwrap();
        let vertex_shader_stage = vk::PipelineShaderStageCreateInfo::builder()
            .stage(vk::ShaderStageFlags::VERTEX)
            .module(*vertex_shader_module)
            .name(&main_function_name);
        let fragment_shader_stage = vk::PipelineShaderStageCreateInfo::builder()
            .stage(vk::ShaderStageFlags::FRAGMENT)
            .module(*fragment_shader_module)
            .name(&main_function_name);
        let shader_stages =
            vec![vertex_shader_stage.build(), fragment_shader_stage.build()];

        // Vertex buffer
        let vbo_wrapper = resource_manager
            .get_item::<BufferWrapper>(
                Handle::with_unique_id(vbo_index as u32, 0))
            .unwrap();
        let vbo_handle = vbo_wrapper.buffer;

        // Vertex input configuration
        let vertex_attrib_descriptions = [
            vk::VertexInputAttributeDescription {
                binding: 0,
                location: 0,
                offset: 0,
                format: vk::Format::R32G32B32_SFLOAT
            },
            vk::VertexInputAttributeDescription {
                binding: 0,
                location: 1,
                offset: 12,
                format: vk::Format::R32G32B32_SFLOAT
            },
            vk::VertexInputAttributeDescription {
                binding: 0,
                location: 2,
                offset: 24,
                format: vk::Format::R32G32_SFLOAT
            }
        ];
        let vertex_binding_descriptions = [
            vk::VertexInputBindingDescription {
                binding: 0,
                stride: vbo_stride_bytes,
                input_rate: vk::VertexInputRate::VERTEX
            }
        ];
        let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::builder()
            .vertex_attribute_descriptions(&vertex_attrib_descriptions)
            .vertex_binding_descriptions(&vertex_binding_descriptions);
        let input_assembly_info = vk::PipelineInputAssemblyStateCreateInfo::builder()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST);

        // Create uniform buffer
        let uniform_buffer = {
            let uniform_buffer_data: Vec<u8> = vec![0; ubo_size_bytes];
            let creation_data = VboCreationData {
                vertex_data: Some(uniform_buffer_data.as_ptr()),
                vertex_size_bytes: std::mem::size_of::<u8>(),
                vertex_count: ubo_size_bytes,
                draw_indexed: false,
                index_data: None,
                usage: BufferUsage::UniformBuffer
            };
            let buffer = BufferWrapper::create(
                context,
                resource_manager,
                &creation_data)?;
            buffer
        };

        // Texture image
        //TODO - Vec from texture_indices.iter().map(|index| ...).collect()
        let texture_image_view = resource_manager
            .get_item::<ImageWrapper>(
                Handle::with_unique_id(texture_index as u32, 0))
            .unwrap()
            .image_view;

        // Samplers
        let sampler_info = vk::SamplerCreateInfo::builder()
            .min_filter(vk::Filter::LINEAR)
            .mag_filter(vk::Filter::LINEAR);
        let sampler: vk::Sampler = //TODO - Vec from texture_image_views.iter().map(|_| ...).collect()
            context.device
                .create_sampler(&sampler_info, None)
                .map_err(|e| VkError::OpFailed(format!("Error creating sampler: {:?}", e)))?;

        // All the stuff around descriptors
        let pool_sizes = [
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::UNIFORM_BUFFER,
                descriptor_count: 1
            },
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                descriptor_count: 1 //TODO - texture_image_views.len() as u32
            }
        ];
        let descriptor_pool_info = vk::DescriptorPoolCreateInfo::builder()
            .max_sets(1)
            .pool_sizes(&pool_sizes);
        let descriptor_pool = context.device
            .create_descriptor_pool(&descriptor_pool_info, None)
            .map_err(|e|
                VkError::OpFailed(format!("Error creating descriptor pool: {:?}", e))
            )?;
        let descriptor_layouts = vec![*descriptor_set_layout];
        let descriptor_set_alloc_info = vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(descriptor_pool)
            .set_layouts(&descriptor_layouts);
        let descriptor_set = context.device
            .allocate_descriptor_sets(&descriptor_set_alloc_info)
            .map_err(|e|
                VkError::OpFailed(format!("Failed allocating descriptor sets: {:?}", e))
            )?
            [0];

        // Descriptor bindings
        let buffer_infos = [vk::DescriptorBufferInfo {
            buffer: uniform_buffer.buffer(),
            offset: 0,
            range: ubo_size_bytes as u64
        }];
        // TODO - (0..texture_image_views.len()).map(|index| vk_renderer::DescriptorImageInfo with texture_image_views[index]).collect()
        let image_infos = [vk::DescriptorImageInfo {
            image_view: texture_image_view,
            sampler: sampler,
            image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL
        }];
        let descriptor_set_writes: Vec<vk::WriteDescriptorSet> = {
            let mut writes = vec![vk::WriteDescriptorSet::builder()
                .dst_set(descriptor_set)
                .dst_binding(0)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .buffer_info(&buffer_infos)
                .build()];
            // TODO - foreach index in texture_image_views, push with binding 1 + index
            writes.push(vk::WriteDescriptorSet::builder()
                .dst_set(descriptor_set)
                .dst_binding(1)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .image_info(&image_infos)
                .build());
            writes
        };
        context.device.update_descriptor_sets(
            &descriptor_set_writes.as_slice(),
            &[]);

        // Viewport
        let viewports = [vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: render_extent.width as f32,
            height: render_extent.height as f32,
            min_depth: 0.0,
            max_depth: 1.0
        }];
        let scissors = [vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: render_extent
        }];
        let viewport_info = vk::PipelineViewportStateCreateInfo::builder()
            .viewports(&viewports)
            .scissors(&scissors);

        // Random pipeline configurations
        let rasterizer_info = vk::PipelineRasterizationStateCreateInfo::builder()
            .line_width(1.0)
            .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
            .cull_mode(vk::CullModeFlags::BACK)
            .polygon_mode(vk::PolygonMode::FILL);
        let multisampler_info = vk::PipelineMultisampleStateCreateInfo::builder()
            .rasterization_samples(vk::SampleCountFlags::TYPE_1);
        let depth_stencil_info = vk::PipelineDepthStencilStateCreateInfo::builder()
            .depth_test_enable(true)
            .depth_write_enable(true)
            .depth_compare_op(vk::CompareOp::LESS_OR_EQUAL);
        let colour_blend_attachments = [
            vk::PipelineColorBlendAttachmentState::builder()
                .blend_enable(true)
                .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
                .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
                .color_blend_op(vk::BlendOp::ADD)
                .src_alpha_blend_factor(vk::BlendFactor::SRC_ALPHA)
                .dst_alpha_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
                .alpha_blend_op(vk::BlendOp::ADD)
                .color_write_mask(vk::ColorComponentFlags::RGBA)
                .build()
        ];
        let colour_blend_info = vk::PipelineColorBlendStateCreateInfo::builder()
            .attachments(&colour_blend_attachments);

        // Make pipeline
        let pipeline_create_info = vk::GraphicsPipelineCreateInfo::builder()
            .stages(&shader_stages)
            .vertex_input_state(&vertex_input_info)
            .input_assembly_state(&input_assembly_info)
            .viewport_state(&viewport_info)
            .rasterization_state(&rasterizer_info)
            .multisample_state(&multisampler_info)
            .depth_stencil_state(&depth_stencil_info)
            .color_blend_state(&colour_blend_info)
            .layout(*pipeline_layout)
            .render_pass(renderpass_wrapper.renderpass)
            .subpass(0);
        let graphics_pipeline = context.device
            .create_graphics_pipelines(
                vk::PipelineCache::null(),
                &[pipeline_create_info.build()],
                None)
            .map_err(|e|
                VkError::OpFailed(format!("{:?}", e))
            )?;

        self.vertex_buffer = vbo_handle;
        self.vertex_count = vbo_wrapper.element_count;
        self.uniform_buffer = uniform_buffer;
        self.texture_image_view = texture_image_view; // TODO - Vec
        self.sampler = sampler; // TODO - Vec
        self.descriptor_pool = descriptor_pool;
        self.descriptor_set = descriptor_set;
        self.pipeline = graphics_pipeline[0];

        Ok(())
    }

    /// Record the commands to render this step; assume that beginning/ending the renderpass is
    /// done separately
    pub unsafe fn record_commands(
        &self,
        command_buffer: vk::CommandBuffer,
        context: &VkContext,
        pipeline_layout: vk::PipelineLayout
    ) {
        context.device.cmd_bind_pipeline(
            command_buffer,
            vk::PipelineBindPoint::GRAPHICS,
            self.pipeline);
        context.device.cmd_bind_vertex_buffers(
            command_buffer,
            0,
            &[self.vertex_buffer],
            &[0]);
        context.device.cmd_bind_descriptor_sets(
            command_buffer,
            vk::PipelineBindPoint::GRAPHICS,
            pipeline_layout,
            0,
            &[self.descriptor_set],
            &[]);
        context.device.cmd_draw(
            command_buffer,
            self.vertex_count as u32,
            1,
            0,
            0);
    }

    /// Update the uniform buffer for this step from the supplied pointer and data size
    pub unsafe fn update_uniform_buffer(
        &self,
        context: &VkContext,
        data_ptr: *const u8,
        size_bytes: usize
    ) -> Result<(), VkError> {
        let (allocator, _) = context.get_mem_allocator();
        self.uniform_buffer.update::<u8>(
            allocator,
            0,
            data_ptr,
            size_bytes)
    }
}
