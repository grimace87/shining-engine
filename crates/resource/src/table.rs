
use crate::{Handle, ResourceLoader, Resource};
use std::any::Any;

pub trait DynamicTable<L: ResourceLoader> {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn free_all_resources(&mut self, loader: &L);
}

pub struct HandleTable<T: 'static> {
    pub(crate) next_index_guess: u32,
    next_unique_id: u32,
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

    pub(crate) fn new() -> Self {
        Self {
            next_index_guess: 0,
            next_unique_id: 1,
            items: vec![]
        }
    }

    pub(crate) fn push_new_resource(&mut self, item: T) -> Handle {
        let table_index = self.obtain_next_index();
        self.items[table_index as usize] = Some(item);
        Handle::for_resource(table_index)
    }

    pub(crate) fn push_new_with_handle(&mut self, handle: Handle, item: T) {

        let table_index = handle.table_index() as usize;

        // If vector doesn't yet have the index
        if table_index >= self.items.len() {
            let extra_length = table_index as usize + 1 - self.items.len();
            for _ in 0..extra_length {
                self.items.push(None);
            }
            self.items[table_index] = Some(item);
            return;
        }

        // Vector had the index already; it must be unused
        if self.items[table_index].is_none() {
            self.items[table_index] = Some(item);
            return;
        }

        panic!("Tried to push a new handle which was already taken!");
    }

    pub(crate) fn remove(&mut self, handle: Handle) -> Option<T> {
        let table_index = handle.table_index() as usize;
        if table_index >= self.items.len() {
            return None;
        }
        if self.items[table_index].is_some() {
            self.next_index_guess = table_index as u32;
        }
        self.items[table_index].take()
    }

    pub fn query_handle(&self, handle: Handle) -> Option<&T> {
        if let Some(item) = &self.items[handle.table_index() as usize] {
            return Some(item);
        }
        None
    }

    fn obtain_next_index(&mut self) -> u32 {

        // Check if index is outside of current vector size; guaranteed unused
        if self.next_index_guess >= self.items.len() as u32 {
            let index = self.next_index_guess;
            let extra_length = self.next_index_guess as usize + 1 - self.items.len();
            for _ in 0..extra_length {
                self.items.push(None);
            }
            self.next_index_guess = self.next_index_guess + 1;
            return index;
        }

        // Check slot is unused
        if self.items[self.next_index_guess as usize].is_none() {
            let index = self.next_index_guess;
            self.next_index_guess = index + 1;
            return index;
        }

        // Need to find an unused slot
        for slot in 0..self.items.len() {
            if self.items[slot].is_none() {
                let index = slot as u32;
                self.next_index_guess = index + 1;
                return index;
            }
        }

        // No unused slot found; add to the end
        let index = self.items.len() as u32;
        self.next_index_guess = index + 1;
        self.items.push(None);
        index
    }
}
