mod handle;
mod manager;
mod resource_types;
mod table;

pub use handle::Handle;
pub use manager::EcsManager;
pub use table::{HandleTable, DynamicTable};

pub mod resource {
    use crate::resource_types;
    pub use resource_types::{Resource, RawResourceBearer, ResourceLoader};
}

#[cfg(test)]
mod tests;
