
use crate::{EcsManager, resource::ResourceLoader};

pub trait RawResourceBearer<L: ResourceLoader> {

    fn initialise_static_resources(
        &self,
        ecs: &mut EcsManager<L>,
        loader: &L
    ) -> Result<(), L::LoadError>;

    fn reload_dynamic_resources(
        &self,
         ecs: &mut EcsManager<L>,
        loader: &mut L,
        swapchain_image_count: usize
    ) -> Result<(), L::LoadError>;
}
