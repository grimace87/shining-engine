mod bearer;
mod handle;
mod loader;
mod manager;
mod table;

pub use handle::{Handle, HandleInterface};
pub use manager::ResourceManager;
pub use loader::{ResourceLoader, null::NullResourceLoader};
pub use bearer::RawResourceBearer;
pub use table::{HandleTable, DynamicTable};

pub trait Resource<L: ResourceLoader>: Sized + 'static {
    type CreationData;
    fn create(
        loader: &L,
        resource_manager: &ResourceManager<L>,
        data: &Self::CreationData
    ) -> Result<Self, L::LoadError>;
    fn release(&self, loader: &L);
}

#[cfg(test)]
mod tests;
