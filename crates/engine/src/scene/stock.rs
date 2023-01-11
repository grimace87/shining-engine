use crate::{Renderable, Scene, SceneFactory, StockRenderable};

pub struct StockScene {
    vbo_size_bytes: usize
}

impl StockScene {
    pub fn new(vertex_count: usize) -> Self {
        Self { vbo_size_bytes: vertex_count * std::mem::size_of::<model::StaticVertex>() }
    }
}

impl Scene for StockScene {
    fn get_renderable(&self) -> Box<dyn Renderable> {
        Box::new(StockRenderable::new(self.vbo_size_bytes))
    }
}

pub struct StockSceneFactory {
    vertex_count: usize
}

impl StockSceneFactory {
    pub fn new(vertex_count: usize) -> Self {
        Self { vertex_count }
    }
}

impl SceneFactory for StockSceneFactory {
    fn get_scene(&self) -> Box<dyn Scene> {
        Box::new(StockScene::new(self.vertex_count))
    }
}
