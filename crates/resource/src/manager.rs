
use crate::{ResourceLoader, RawResourceBearer};
use std::collections::HashMap;

pub struct ResourceManager<L: ResourceLoader> {
    loaded_models: HashMap<u32, L::VertexBufferHandle>,
    loaded_textures: HashMap<u32, L::TextureHandle>
}

impl<L: ResourceLoader> ResourceManager<L> {

    pub fn new() -> Self {
        Self {
            loaded_models: HashMap::new(),
            loaded_textures: HashMap::new()
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
            let loaded_model = loader.load_model(&raw_data)?;
            self.loaded_models.insert(*id, loaded_model);
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

        Ok(())
    }

    pub fn get_vbo_handle(&self, id: u32) -> Result<&L::VertexBufferHandle, L::LoadError> {
        self.loaded_models.get(&id)
            .ok_or_else(|| L::make_error(format!("Error: Model {} is not loaded", id)))
    }

    pub fn get_texture_handle(&self, id: u32) -> Result<&L::TextureHandle, L::LoadError> {
        self.loaded_textures.get(&id)
            .ok_or_else(|| L::make_error(format!("Error: Texture {} is not loaded", id)))
    }
}
