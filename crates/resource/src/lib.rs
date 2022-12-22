mod manager;

pub use manager::ResourceManager;

use model::StaticVertex;
use std::collections::HashMap;

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

/// VboCreationData struct
/// Specification for how a vertex buffer is to be created
pub struct VboCreationData {
    pub vertex_data: Vec<StaticVertex>,
    pub vertex_count: usize,
    pub draw_indexed: bool,
    pub index_data: Option<Vec<u16>>
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

/// ResourcePreloads struct
/// Encapsulates everything needed to initialise all of the resources that need to be preloaded in
/// order to render a scene.
pub struct ResourcePreloads {
    pub vbo_preloads: HashMap<usize, VboCreationData>,
    pub texture_preloads: HashMap<usize, TextureCreationData>
}

pub trait ResourceLoader {

    type VertexBufferHandle;
    type TextureHandle;
    type LoadError;

    fn load_model(&self, raw_data: &VboCreationData) -> Result<(Self::VertexBufferHandle, usize), Self::LoadError>;
    fn release_model(&mut self, model: &Self::VertexBufferHandle) -> Result<(), Self::LoadError>;

    fn load_texture(&self, raw_data: &TextureCreationData) -> Result<Self::TextureHandle, Self::LoadError>;
    fn release_texture(&mut self, texture: &Self::TextureHandle) -> Result<(), Self::LoadError>;

    fn make_error(message: String) -> Self::LoadError;
}

pub trait RawResourceBearer {

    fn get_model_resource_ids(&self) -> &[u32];
    fn get_texture_resource_ids(&self) -> &[u32];

    fn get_raw_model_data(&self, id: u32) -> VboCreationData;
    fn get_raw_texture_data(&self, id: u32) -> TextureCreationData;
}