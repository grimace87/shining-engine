mod bearer;
mod loader;

pub use bearer::RawResourceBearer;
pub use loader::ResourceLoader;
use crate::EcsManager;

pub trait Resource<L: ResourceLoader>: Sized + 'static {
    type CreationData;
    fn create(
        loader: &L,
        ecs: &EcsManager<L>,
        data: &Self::CreationData
    ) -> Result<Self, L::LoadError>;
    fn release(&self, loader: &L);
}
