
use crate::{Handle, ResourceLoader, Resource};
use std::any::Any;

pub trait DynamicTable<L: ResourceLoader> {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn free_all_resources(&mut self, loader: &L);
}

pub struct HandleTable<T: 'static> {
    pub(crate) next_handle_guess: Handle,
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
            next_handle_guess: 1,
            items: vec![]
        }
    }

    pub(crate) fn push_new(&mut self, item: T) -> Handle {
        let handle = self.obtain_next_handle();
        self.items[handle as usize] = Some(item);
        handle
    }

    pub(crate) fn push_new_with_handle(&mut self, handle: Handle, item: T) {

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

    pub(crate) fn remove(&mut self, handle: Handle) -> Option<T> {
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
