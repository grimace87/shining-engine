
use crate::{Handle, DynamicTable, HandleTable, ResourceLoader, Resource};

pub struct ResourceManager<L: ResourceLoader> {
    tables: Vec<Box<dyn DynamicTable<L>>>
}

impl<L: ResourceLoader> ResourceManager<L> {

    pub fn new() -> Self {
        Self {
            tables: vec![]
        }
    }

    pub(crate) fn next_index_guess<T: Resource<L>>(&self) -> Option<u32> {
        for table in self.tables.iter() {
            if let Some(table) = table.as_any().downcast_ref::<HandleTable<T>>() {
                return Some(table.next_index_guess);
            }
        }
        None
    }

    pub fn add_item<T: Resource<L>>(
        &mut self,
        item: T
    ) -> Handle {

        for table in self.tables.iter_mut() {
            if let Some(table) = table.as_any_mut().downcast_mut::<HandleTable<T>>() {
                let handle = table.push_new_resource(item);
                return handle;
            }
        }

        let mut table = HandleTable::new();
        let handle = table.push_new_resource(item);
        self.tables.push(Box::new(table));
        handle
    }

    pub fn push_new_with_handle<T: Resource<L>>(&mut self, handle: Handle, item: T) {

        for table in self.tables.iter_mut() {
            if let Some(table) = table.as_any_mut().downcast_mut::<HandleTable<T>>() {
                table.push_new_with_handle(handle, item);
                return;
            }
        }
    }

    pub fn get_item<T: Resource<L>>(&self, handle: Handle) -> Option<&T> {
        for table in self.tables.iter() {
            if let Some(table) = table.as_any().downcast_ref::<HandleTable<T>>() {
                return table.query_handle(handle);
            }
        }
        None
    }

    pub fn remove_item<T: Resource<L>>(
        &mut self,
        handle: Handle
    ) -> Option<T> {
        for table in self.tables.iter_mut() {
            if let Some(table) = table.as_any_mut().downcast_mut::<HandleTable<T>>() {
                return table.remove(handle);
            }
        }
        None
    }

    pub fn free_all_resources(&mut self, loader: &L) -> Result<(), L::LoadError> {

        for table in self.tables.iter_mut() {
            table.free_all_resources(loader);
        }

        self.tables.clear();

        Ok(())
    }
}
