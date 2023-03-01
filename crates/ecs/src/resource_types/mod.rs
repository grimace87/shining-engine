mod bearer;

pub use bearer::RawResourceBearer;
use crate::EcsManager;
use error::EngineError;

pub trait Resource<L>: Sized + 'static {
    type CreationData;
    fn create(
        loader: &L,
        ecs: &EcsManager<L>,
        data: &Self::CreationData
    ) -> Result<Self, EngineError>;
    fn release(&self, loader: &L);
}
