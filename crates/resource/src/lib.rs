mod manager;

pub use manager::ResourceManager;

use model::StaticVertex;

/// ImageUsage enum
/// An enumeration of what purpose buffer resources can be used for
#[derive(PartialEq, Debug)]
pub enum BufferUsage {
    InitialiseOnceVertexBuffer,
    UniformBuffer
}

/// VboCreationData struct
/// Specification for how a vertex buffer is to be created
pub struct VboCreationData {
    pub vertex_data: Vec<StaticVertex>,
    pub vertex_count: usize,
    pub draw_indexed: bool,
    pub index_data: Option<Vec<u16>>,
    pub usage: BufferUsage
}

/// TexturePixelFormat enum
/// Abstraction of the set of pixel formats known by the engine
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum TexturePixelFormat {
    None,
    Rgba,
    Unorm16
}

/// ImageUsage enum
/// An enumeration of what purpose image resources can be used for
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum ImageUsage {
    TextureSampleOnly,
    DepthBuffer,
    OffscreenRenderSampleColorWriteDepth,
    Skybox
}

/// TextureCreationData struct
/// Specification for how a texture resource is to be created
pub struct TextureCreationData {
    pub layer_data: Option<Vec<Vec<u8>>>,
    pub width: u32,
    pub height: u32,
    pub format: TexturePixelFormat,
    pub usage: ImageUsage
}

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

/// OffscreenFramebufferData struct
/// Information needed to prepare a non-swapchain framebuffer
pub struct OffscreenFramebufferData {
    pub width: u32,
    pub height: u32,
    pub color_format: TexturePixelFormat,
    pub depth_format: TexturePixelFormat
}

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

impl RenderpassCreationData {

    /// Checking for whether this complex ID targets the swapchain.
    /// This would indicate that the resource is no longer valid after the swapchain has been
    /// recreated.
    pub fn id_uses_swapchain(id: u64) -> bool {
        let using_swapchain_bit = id & 0x0001000000000000;
        using_swapchain_bit != 0
    }

    pub fn extract_id(complex_id: u64) -> u32 {
        (complex_id & 0x0000ffff) as u32
    }

    pub fn encode_complex_renderpass_id(
        &self,
        id: u32,
        current_swapchain_width: u32,
        current_swapchain_height: u32
    ) -> u64 {
        if id > 0x0000ffff {
            panic!("Renderpass ID cannot be greater than 65535");
        }
        let id_bits = (id as u64) & 0x0000ffff;
        let width_bits = match self.target {
            RenderpassTarget::SwapchainImageWithDepth => {
                ((current_swapchain_width & 0x0000ffff) as u64) << 16
            },
            RenderpassTarget::OffscreenImageWithDepth(_, target_width, _) => {
                ((target_width & 0x0000ffff) as u64) << 16
            }
        };
        let height_bits = match self.target {
            RenderpassTarget::SwapchainImageWithDepth => {
                ((current_swapchain_height & 0x0000ffff) as u64) << 32
            },
            RenderpassTarget::OffscreenImageWithDepth(_, _, target_height) => {
                ((target_height & 0x0000ffff) as u64) << 32
            }
        };
        let using_swapchain_bit: u64 = match self.target {
            RenderpassTarget::SwapchainImageWithDepth => 0x1 << 48,
            RenderpassTarget::OffscreenImageWithDepth(_, _, _) => 0x0 << 48
        };
        let swapchain_index_bits = ((self.swapchain_image_index & 0x00000007) as u64) << 49;
        id_bits | width_bits | height_bits | using_swapchain_bit | swapchain_index_bits
    }
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

impl PipelineCreationData {

    pub fn extract_renderpass_id(complex_id: u64) -> u32 {
        ((complex_id >> 16) as u32) & 0x0000ffff
    }

    pub fn encode_complex_pipeline_id(&self, id: u32) -> u64 {
        if id > 0x0000ffff {
            panic!("Pipeline ID cannot be greater than 65535");
        }
        let id_bits = (id as u64) & 0x0000ffff;
        let renderpass_id_bits = ((self.renderpass_index & 0x0000ffff) as u64) << 16;
        let swapchain_index_bits = ((self.swapchain_image_index & 0x0000000f) as u64) << 32;
        id_bits | renderpass_id_bits | swapchain_index_bits
    }
}

pub trait ResourceLoader where Self: Sized {

    type VertexBufferHandle;
    type TextureHandle;
    type ShaderHandle;
    type OffscreenFramebufferHandle;
    type RenderpassHandle;
    type DescriptorSetLayoutHandle;
    type PipelineLayoutHandle;
    type PipelineHandle;
    type LoadError;

    fn load_model(
        &self,
        raw_data: &VboCreationData
    ) -> Result<(Self::VertexBufferHandle, usize), Self::LoadError>;
    fn release_model(
        &mut self,
        model: &Self::VertexBufferHandle
    ) -> Result<(), Self::LoadError>;

    fn load_texture(
        &self,
        raw_data: &TextureCreationData
    ) -> Result<Self::TextureHandle, Self::LoadError>;
    fn release_texture(
        &mut self,
        texture: &Self::TextureHandle
    ) -> Result<(), Self::LoadError>;

    fn load_shader(
        &self,
        raw_data: &ShaderCreationData
    ) -> Result<Self::ShaderHandle, Self::LoadError>;
    fn release_shader(
        &mut self,
        shader: &Self::ShaderHandle
    ) -> Result<(), Self::LoadError>;

    fn load_offscreen_framebuffer(
        &self,
        raw_data: &OffscreenFramebufferData
    ) -> Result<Self::OffscreenFramebufferHandle, Self::LoadError>;
    fn release_offscreen_framebuffer(
        &mut self,
        framebuffer: &Self::OffscreenFramebufferHandle
    ) -> Result<(), Self::LoadError>;

    fn load_renderpass(
        &self,
        raw_data: &RenderpassCreationData,
        resource_manager: &ResourceManager<Self>
    ) -> Result<Self::RenderpassHandle, Self::LoadError>;
    fn release_renderpass(
        &mut self,
        renderpass: &Self::RenderpassHandle
    ) -> Result<(), Self::LoadError>;

    fn load_descriptor_set_layout(
        &self,
        raw_data: &DescriptorSetLayoutCreationData
    ) -> Result<Self::DescriptorSetLayoutHandle, Self::LoadError>;
    fn release_descriptor_set_layout(
        &mut self,
        descriptor_set_layout: &Self::DescriptorSetLayoutHandle
    ) -> Result<(), Self::LoadError>;

    fn load_pipeline_layout(
        &self,
        raw_data: &PipelineLayoutCreationData,
        resource_manager: &ResourceManager<Self>
    ) -> Result<Self::PipelineLayoutHandle, Self::LoadError>;
    fn release_pipeline_layout(
        &mut self,
        pipeline_layout: &Self::PipelineLayoutHandle
    ) -> Result<(), Self::LoadError>;

    fn load_pipeline(
        &self,
        raw_data: &PipelineCreationData,
        resource_manager: &ResourceManager<Self>,
        current_swapchain_width: u32,
        current_swapchain_height: u32
    ) -> Result<Self::PipelineHandle, Self::LoadError>;
    fn release_pipeline(
        &mut self,
        pipeline: &Self::PipelineHandle
    ) -> Result<(), Self::LoadError>;

    fn make_error(message: String) -> Self::LoadError;
}

pub trait RawResourceBearer {

    fn get_model_resource_ids(&self) -> &[u32];
    fn get_texture_resource_ids(&self) -> &[u32];
    fn get_shader_resource_ids(&self) -> &[u32];
    fn get_offscreen_framebuffer_resource_ids(&self) -> &[u32];
    fn get_renderpass_resource_ids(&self) -> &[u32];
    fn get_descriptor_set_layout_resource_ids(&self) -> &[u32];
    fn get_pipeline_layout_resource_ids(&self) -> &[u32];
    fn get_pipeline_resource_ids(&self) -> &[u32];

    fn get_raw_model_data(&self, id: u32) -> VboCreationData;
    fn get_raw_texture_data(&self, id: u32) -> TextureCreationData;
    fn get_raw_shader_data(&self, id: u32) -> ShaderCreationData;
    fn get_raw_offscreen_framebuffer_data(&self, id: u32) -> OffscreenFramebufferData;
    fn get_raw_renderpass_data(
        &self, id: u32, swapchain_image_index: usize) -> RenderpassCreationData;
    fn get_raw_descriptor_set_layout_data(&self, id: u32) -> DescriptorSetLayoutCreationData;
    fn get_raw_pipeline_layout_data(&self, id: u32) -> PipelineLayoutCreationData;
    fn get_raw_pipeline_data(
        &self, id: u32, swapchain_image_index: usize) -> PipelineCreationData;
}
