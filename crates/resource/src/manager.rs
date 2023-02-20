
use crate::ResourceLoader;
use std::any::Any;

pub type Handle = u64;

pub trait HandleInterface {
    type HandleComponent;
    fn from_parts(handle: Self::HandleComponent, id: Self::HandleComponent) -> Self;
    fn handle_part(&self) -> Self::HandleComponent;
    fn id_part(&self) -> Self::HandleComponent;
}

impl HandleInterface for Handle {
    type HandleComponent = u32;

    #[inline]
    fn from_parts(handle: u32, id: u32) -> Handle {
        ((id as u64) << 32) | handle as u64
    }

    #[inline]
    fn handle_part(&self) -> u32 {
        (self & 0xffffffff) as u32
    }

    #[inline]
    fn id_part(&self) -> u32 {
        (self >> 32) as u32
    }
}

pub struct ResourceManager<L: ResourceLoader> {
    tables: Vec<Box<dyn DynamicTable<L>>>
}

impl<L: ResourceLoader> ResourceManager<L> {

    pub fn new() -> Self {
        Self {
            tables: vec![]
        }
    }

    pub fn add_item<T: Resource<L>>(
        &mut self,
        item: T
    ) -> Handle {

        for table in self.tables.iter_mut() {
            if let Some(table) = table.as_any_mut().downcast_mut::<HandleTable<T>>() {
                let handle = table.push_new(item);
                return handle;
            }
        }

        let mut table = HandleTable::new();
        let handle = table.push_new(item);
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
                return table.items
                    .get(handle as usize)
                    .unwrap_or(&None)
                    .as_ref();
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

pub trait Resource<L: ResourceLoader>: Sized + 'static {
    type CreationData;
    fn create(
        loader: &L,
        resource_manager: &ResourceManager<L>,
        data: &Self::CreationData
    ) -> Result<Self, L::LoadError>;
    fn release(&self, loader: &L);
}

pub trait DynamicTable<L: ResourceLoader> {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn free_all_resources(&mut self, loader: &L);
}

pub struct HandleTable<T: 'static> {
    next_handle_guess: Handle,
    items: Vec<Option<T>>
}

impl<L: ResourceLoader, T: Resource<L> + 'static> DynamicTable<L> for HandleTable<T> {

    fn as_any(&self) -> &dyn Any {
        self as &dyn Any
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self as &mut dyn Any
    }

    fn free_all_resources(&mut self, loader: &L) {
        for item in self.items.iter() {
            if let Some(item) = item {
                item.release(loader);
            }
        }
        self.items.clear();
    }
}

impl<T: 'static> HandleTable<T> {

    fn new() -> Self {
        Self {
            next_handle_guess: 1,
            items: vec![]
        }
    }

    fn push_new(&mut self, item: T) -> Handle {
        let handle = self.obtain_next_handle();
        self.items[handle as usize] = Some(item);
        handle
    }

    fn push_new_with_handle(&mut self, handle: Handle, item: T) {

        // If vector doesn't yet have the index
        if handle as usize >= self.items.len() {
            let extra_length = self.next_handle_guess as usize + 1 - self.items.len();
            for _ in 0..extra_length {
                self.items.push(None);
            }
            self.items[handle as usize] = Some(item);
            return;
        }

        // Vector had the index already; it must be unused
        if self.items[handle as usize].is_none() {
            self.items[handle as usize] = Some(item);
            return;
        }

        panic!("Tried to push a new handle which was already taken!");
    }

    fn remove(&mut self, handle: Handle) -> Option<T> {
        if self.items[handle as usize].is_some() {
            self.next_handle_guess = handle;
        }
        self.items[handle as usize].take()
    }

    pub fn query_handle(&self, handle: Handle) -> Option<&T> {
        if let Some(item) = &self.items[handle as usize] {
            return Some(item);
        }
        None
    }

    fn obtain_next_handle(&mut self) -> Handle {

        // Check handle is outside of current vector size; guaranteed unused
        if self.next_handle_guess >= self.items.len() as Handle {
            let handle = self.next_handle_guess;
            let extra_length = self.next_handle_guess as usize + 1 - self.items.len();
            for _ in 0..extra_length {
                self.items.push(None);
            }
            self.next_handle_guess = self.next_handle_guess + 1;
            return handle;
        }

        // Check slot is unused
        if self.items[self.next_handle_guess as usize].is_none() {
            let handle = self.next_handle_guess;
            self.next_handle_guess = handle + 1;
            return handle;
        }

        // Need to find an unused slot
        for slot in 0..self.items.len() {
            if self.items[slot].is_none() {
                let handle = slot as Handle;
                self.next_handle_guess = handle + 1;
                return handle;
            }
        }

        // No unused slot found; add to the end
        let handle = self.items.len() as Handle;
        self.next_handle_guess = handle + 1;
        self.items.push(None);
        handle
    }
}
