
use crate::{ResourceLoader, RawResourceBearer};
use std::collections::HashMap;

pub struct ResourceManager<L: ResourceLoader> {
    loaded_models: HashMap<u32, (L::VertexBufferHandle, usize)>,
    loaded_textures: HashMap<u32, L::TextureHandle>,
    loaded_shaders: HashMap<u32, L::ShaderHandle>
}

impl<L: ResourceLoader> ResourceManager<L> {

    pub fn new() -> Self {
        Self {
            loaded_models: HashMap::new(),
            loaded_textures: HashMap::new(),
            loaded_shaders: HashMap::new()
        }
    }

    pub fn load_resources_from<B: RawResourceBearer>(
        &mut self,
        loader: &L,
        bearer: &B
    ) -> Result<(), L::LoadError> {

        let model_ids = bearer.get_model_resource_ids();
        for id in model_ids {
            if self.loaded_models.contains_key(id) {
                continue;
            }
            let raw_data = bearer.get_raw_model_data(*id);
            let (model_data, vertex_count) = loader.load_model(&raw_data)?;
            self.loaded_models.insert(*id, (model_data, vertex_count));
        }

        let texture_ids = bearer.get_texture_resource_ids();
        for id in texture_ids {
            if self.loaded_textures.contains_key(id) {
                continue;
            }
            let raw_data = bearer.get_raw_texture_data(*id);
            let loaded_texture = loader.load_texture(&raw_data)?;
            self.loaded_textures.insert(*id, loaded_texture);
        }

        let shader_ids = bearer.get_shader_resource_ids();
        for id in shader_ids {
            if self.loaded_shaders.contains_key(id) {
                continue;
            }
            let raw_data = bearer.get_raw_shader_data(*id);
            let loaded_shader = loader.load_shader(&raw_data)?;
            self.loaded_shaders.insert(*id, loaded_shader);
        }

        Ok(())
    }

    pub fn free_resources(&mut self, loader: &mut L) -> Result<(), L::LoadError> {

        for (_, (handle, _)) in self.loaded_models.iter() {
            loader.release_model(handle)?;
        }
        self.loaded_models.clear();

        for (_, handle) in self.loaded_textures.iter() {
            loader.release_texture(handle)?;
        }
        self.loaded_textures.clear();

        for (_, handle) in self.loaded_shaders.iter() {
            loader.release_shader(handle)?;
        }
        self.loaded_shaders.clear();

        Ok(())
    }

    pub fn get_vbo_handle(&self, id: u32) -> Result<(&L::VertexBufferHandle, usize), L::LoadError> {
        let (model_data, vertex_count) = self.loaded_models.get(&id)
            .ok_or_else(|| L::make_error(format!("Error: Model {} is not loaded", id)))?;
        Ok((model_data, *vertex_count))
    }

    pub fn get_texture_handle(&self, id: u32) -> Result<&L::TextureHandle, L::LoadError> {
        self.loaded_textures.get(&id)
            .ok_or_else(|| L::make_error(format!("Error: Texture {} is not loaded", id)))
    }

    pub fn get_shader_handle(&self, id: u32) -> Result<&L::ShaderHandle, L::LoadError> {
        self.loaded_shaders.get(&id)
            .ok_or_else(|| L::make_error(format!("Error: Shader {} is not loaded", id)))
    }
}
