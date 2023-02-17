
use crate::{ResourceLoader, RawResourceBearer, RenderpassCreationData, PipelineCreationData};
use std::collections::HashMap;
use std::ops::Not;

pub struct ResourceManager<L: ResourceLoader> {
    loaded_models: HashMap<u32, L::VertexBufferHandle>,
    loaded_textures: HashMap<u32, L::TextureHandle>,
    loaded_shaders: HashMap<u32, L::ShaderHandle>,
    loaded_offscreen_framebuffers: HashMap<u32, L::OffscreenFramebufferHandle>,
    loaded_renderpasses: HashMap<u64, L::RenderpassHandle>,
    loaded_descriptor_set_layouts: HashMap<u32, L::DescriptorSetLayoutHandle>,
    loaded_pipeline_layouts: HashMap<u32, L::PipelineLayoutHandle>,
    loaded_pipelines: HashMap<u64, L::PipelineHandle>
}

impl<L: ResourceLoader> ResourceManager<L> {

    pub fn new() -> Self {
        Self {
            loaded_models: HashMap::new(),
            loaded_textures: HashMap::new(),
            loaded_shaders: HashMap::new(),
            loaded_offscreen_framebuffers: HashMap::new(),
            loaded_renderpasses: HashMap::new(),
            loaded_descriptor_set_layouts: HashMap::new(),
            loaded_pipeline_layouts: HashMap::new(),
            loaded_pipelines: HashMap::new()
        }
    }

    /// Load the resources that can be loaded once ahead of time and will never need to be
    /// re-created.
    /// This includes immutable vertex buffers, textures, and static shaders.
    /// These can still be released any time if no longer needed, but there should be no reason
    /// to do this besides keeping memory usage down.
    pub fn load_static_resources_from<T: Sized>(
        &mut self,
        loader: &L,
        bearer: &Box<dyn RawResourceBearer<T>>
    ) -> Result<(), L::LoadError> {

        let model_ids = bearer.get_model_resource_ids();
        for id in model_ids {
            if self.loaded_models.contains_key(id) {
                continue;
            }
            let raw_data = bearer.get_raw_model_data(*id);
            let (model_data, _) = loader.load_model(&raw_data)?;
            self.loaded_models.insert(*id, model_data);
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

        let offscreen_framebuffer_ids = bearer.get_offscreen_framebuffer_resource_ids();
        for id in offscreen_framebuffer_ids {
            if self.loaded_offscreen_framebuffers.contains_key(id) {
                continue;
            }
            let raw_data = bearer.get_raw_offscreen_framebuffer_data(*id);
            let loaded_framebuffer = loader
                .load_offscreen_framebuffer(&raw_data)?;
            self.loaded_offscreen_framebuffers.insert(*id, loaded_framebuffer);
        }

        Ok(())
    }

    /// Load dynamic resources based on requirements of the resource bearer.
    /// These resources may need to be recreated at any time independent of what the app is doing,
    /// and depends more on the running environment. Recreating the Vulkan swapchain will be an
    /// example of a case where many of these resources will need to be recreated.
    pub fn load_dynamic_resources_from<T: Sized>(
        &mut self,
        loader: &L,
        bearer: &Box<dyn RawResourceBearer<T>>,
        swapchain_size: usize,
        current_swapchain_width: u32,
        current_swapchain_height: u32
    ) -> Result<(), L::LoadError> {

        let renderpass_ids = bearer.get_renderpass_resource_ids();
        for id in renderpass_ids {
            for image_index in 0..swapchain_size {
                let raw_data = bearer.get_raw_renderpass_data(*id, image_index);
                let complex_id = raw_data.encode_complex_renderpass_id(
                    *id,
                    current_swapchain_width,
                    current_swapchain_height);
                if self.loaded_renderpasses.contains_key(&complex_id) {
                    continue;
                }
                let loaded_renderpass = loader.load_renderpass(&raw_data, self)?;
                self.loaded_renderpasses.insert(complex_id, loaded_renderpass);
            }
        }

        let descriptor_set_layout_ids = bearer.get_descriptor_set_layout_resource_ids();
        for id in descriptor_set_layout_ids {
            if self.loaded_descriptor_set_layouts.contains_key(id) {
                continue;
            }
            let raw_data = bearer.get_raw_descriptor_set_layout_data(*id);
            let loaded_descriptor_set_layout =
                loader.load_descriptor_set_layout(&raw_data)?;
            self.loaded_descriptor_set_layouts.insert(*id, loaded_descriptor_set_layout);
        }

        let pipeline_layout_ids = bearer.get_pipeline_layout_resource_ids();
        for id in pipeline_layout_ids {
            if self.loaded_pipeline_layouts.contains_key(id) {
                continue;
            }
            let raw_data = bearer.get_raw_pipeline_layout_data(*id);
            let loaded_pipeline_layout =
                loader.load_pipeline_layout(&raw_data, self)?;
            self.loaded_pipeline_layouts.insert(*id, loaded_pipeline_layout);
        }

        let pipeline_ids = bearer.get_pipeline_resource_ids();
        for id in pipeline_ids {
            for image_index in 0..swapchain_size {
                let raw_data = bearer.get_raw_pipeline_data(*id, image_index);
                let complex_id = raw_data.encode_complex_pipeline_id(*id);
                if self.loaded_pipelines.contains_key(&complex_id) {
                    continue;
                }
                let loaded_pipeline = loader.load_pipeline(
                    &raw_data,
                    self,
                    current_swapchain_width,
                    current_swapchain_height)?;
                self.loaded_pipelines.insert(complex_id, loaded_pipeline);
            }
        }

        Ok(())
    }

    /// Release all dynamic resources that can no longer be used once the swapchain has been
    /// recreated. Anything that depended on the images, including framebuffers, will be cleaned
    /// up.
    pub fn release_swapchain_dynamic_resources(
        &mut self,
        loader: &mut L
    ) -> Result<(), L::LoadError> {

        // Find which renderpasses are considered stale based on logic in RenderpassCreationData
        let old_swapchain_ids: Vec<u64> = self.loaded_renderpasses.keys()
            .filter(|&key| {
                RenderpassCreationData::id_uses_swapchain(*key)
            })
            .map(|&key| key)
            .collect();

        // Find any pipelines which will be left dangling by having no corresponding renderpass
        let stale_pipeline_ids: Vec<u64> = self.loaded_pipelines.keys()
            .filter(|&key| {
                let renderpass_id = PipelineCreationData::extract_renderpass_id(*key);
                self.loaded_renderpasses.keys()
                    .any(|key| {
                        let id = RenderpassCreationData::extract_id(*key);
                        renderpass_id == id
                    })
                    .not()
            })
            .map(|&key| key)
            .collect();

        // Release the flagged resources
        for key in stale_pipeline_ids.iter() {
            if let Some(value) = self.loaded_pipelines.get(key) {
                loader.release_pipeline(value)?;
            }
            self.loaded_pipelines.remove(key);
        }
        for key in old_swapchain_ids.iter() {
            if let Some(value) = self.loaded_renderpasses.get(key) {
                loader.release_renderpass(value)?;
            }
            self.loaded_renderpasses.remove(key);
        }

        Ok(())
    }

    pub fn free_resources(&mut self, loader: &mut L) -> Result<(), L::LoadError> {

        for (_, handle) in self.loaded_models.iter() {
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

        for (_, handle) in self.loaded_offscreen_framebuffers.iter() {
            loader.release_offscreen_framebuffer(handle)?;
        }
        self.loaded_offscreen_framebuffers.clear();

        for (_, handle) in self.loaded_renderpasses.iter() {
            loader.release_renderpass(handle)?;
        }
        self.loaded_renderpasses.clear();

        for (_, handle) in self.loaded_descriptor_set_layouts.iter() {
            loader.release_descriptor_set_layout(handle)?;
        }
        self.loaded_descriptor_set_layouts.clear();

        for (_, handle) in self.loaded_pipeline_layouts.iter() {
            loader.release_pipeline_layout(handle)?;
        }
        self.loaded_pipeline_layouts.clear();

        for (_, handle) in self.loaded_pipelines.iter() {
            loader.release_pipeline(handle)?;
        }
        self.loaded_pipelines.clear();

        Ok(())
    }

    pub fn get_vbo_handle(&self, id: u32) -> Result<&L::VertexBufferHandle, L::LoadError> {
        let model_data = self.loaded_models.get(&id)
            .ok_or_else(|| L::make_error(format!("Error: Model {} is not loaded", id)))?;
        Ok(model_data)
    }

    pub fn get_texture_handle(&self, id: u32) -> Result<&L::TextureHandle, L::LoadError> {
        self.loaded_textures.get(&id)
            .ok_or_else(|| L::make_error(format!("Error: Texture {} is not loaded", id)))
    }

    pub fn get_shader_handle(&self, id: u32) -> Result<&L::ShaderHandle, L::LoadError> {
        self.loaded_shaders.get(&id)
            .ok_or_else(|| L::make_error(format!("Error: Shader {} is not loaded", id)))
    }

    pub fn get_offscreen_framebuffer_handle(
        &self,
        id: u32
    ) -> Result<&L::OffscreenFramebufferHandle, L::LoadError> {
        self.loaded_offscreen_framebuffers.get(&id)
            .ok_or_else(|| L::make_error(
                format!("Error: Offscreen framebuffer {} is not loaded", id)
            ))
    }

    pub fn get_renderpass_handle(
        &self,
        complex_id: u64
    ) -> Result<&L::RenderpassHandle, L::LoadError> {
        self.loaded_renderpasses.get(&complex_id)
            .ok_or_else(|| L::make_error(
                format!("Error: Renderpass {} is not loaded", complex_id)
            ))
    }

    pub fn get_descriptor_set_layout_handle(
        &self,
        id: u32
    ) -> Result<&L::DescriptorSetLayoutHandle, L::LoadError> {
        self.loaded_descriptor_set_layouts.get(&id)
            .ok_or_else(|| L::make_error(
                format!("Error: Descriptor set layout {} is not loaded", id)
            ))
    }

    pub fn get_pipeline_layout_handle(
        &self,
        id: u32
    ) -> Result<&L::PipelineLayoutHandle, L::LoadError> {
        self.loaded_pipeline_layouts.get(&id)
            .ok_or_else(|| L::make_error(
                format!("Error: Pipeline layout {} is not loaded", id)
            ))
    }

    pub fn get_pipeline_handle(
        &self,
        complex_id: u64
    ) -> Result<&L::PipelineHandle, L::LoadError> {
        self.loaded_pipelines.get(&complex_id)
            .ok_or_else(|| L::make_error(
                format!("Error: Pipeline {} is not loaded", complex_id)
            ))
    }
}
