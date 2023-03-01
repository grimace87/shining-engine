
use crate::{EcsManager};
use error::EngineError;

pub trait RawResourceBearer<L> {

    fn initialise_static_resources(
        &self,
        ecs: &mut EcsManager<L>,
        loader: &L
    ) -> Result<(), EngineError>;

    fn reload_dynamic_resources(
        &self,
         ecs: &mut EcsManager<L>,
        loader: &mut L,
        swapchain_image_count: usize
    ) -> Result<(), EngineError>;
}
