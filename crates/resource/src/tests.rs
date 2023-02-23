
use crate::{Handle, Resource, NullResourceLoader, ResourceManager};

struct SomeResource;

impl Resource<NullResourceLoader> for SomeResource {
    type CreationData = ();

    fn create(
        _loader: &NullResourceLoader,
        _resource_manager: &ResourceManager<NullResourceLoader>,
        _data: &()
    ) -> Result<Self, String> {
        Ok(SomeResource)
    }

    fn release(&self, _loader: &NullResourceLoader) {}
}

#[test]
fn explicit_handles_can_read_back() {
    let mut manager: ResourceManager<NullResourceLoader> = ResourceManager::new();
    let handle = Handle::for_resource(0x1);
    let resource = SomeResource;

    manager.push_new_with_handle(handle, resource);
    let item_ref = manager.get_item::<SomeResource>(handle);
    assert!(item_ref.is_some());
}

#[test]
fn implicit_handles_count_logically() {
    let mut manager: ResourceManager<NullResourceLoader> = ResourceManager::new();

    let handle_0 = manager.add_item(SomeResource);
    manager.add_item(SomeResource);
    manager.add_item(SomeResource);
    let next_table_index = manager.next_index_guess::<SomeResource>().unwrap();
    assert_eq!(next_table_index, 3);

    manager.remove_item::<SomeResource>(handle_0);
    let next_table_index = manager.next_index_guess::<SomeResource>().unwrap();
    assert_eq!(next_table_index, 0);

    manager.add_item(SomeResource);
    let next_table_index = manager.next_index_guess::<SomeResource>().unwrap();
    assert_eq!(next_table_index, 1);

    manager.add_item(SomeResource);
    let next_table_index = manager.next_index_guess::<SomeResource>().unwrap();
    assert_eq!(next_table_index, 4);
}

#[test]
fn implicit_handles_can_read_back() {
    let mut manager: ResourceManager<NullResourceLoader> = ResourceManager::new();
    let handle_0 = manager.add_item(SomeResource);
    manager.add_item(SomeResource);
    let item_back = manager.remove_item::<SomeResource>(handle_0);
    assert!(item_back.is_some());
}

#[test]
fn unused_handles_read_back_as_none() {
    let mut manager: ResourceManager<NullResourceLoader> = ResourceManager::new();
    manager.add_item(SomeResource);
    manager.add_item(SomeResource);
    let item_back = manager
        .remove_item::<SomeResource>(Handle::for_resource(5));
    assert!(item_back.is_none());
}
