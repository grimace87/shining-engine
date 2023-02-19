
use crate::{ResourceLoader, ResourceManager};

pub trait RawResourceBearer<L: ResourceLoader> {

    fn initialise_static_resources(
        &self,
        manager: &mut ResourceManager<L>,
        loader: &L
    ) -> Result<(), L::LoadError>;

    fn reload_dynamic_resources(
        &self,
        manager: &mut ResourceManager<L>,
        loader: &mut L,
        swapchain_image_count: usize
    ) -> Result<(), L::LoadError>;
}
