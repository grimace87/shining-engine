pub mod stock;

use crate::Renderable;

pub trait SceneFactory {
    fn get_scene(&self) -> Box<dyn Scene>;
}

pub trait Scene {
    fn get_renderable(&self) -> Box<dyn Renderable>;
}
