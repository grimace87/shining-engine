
use crate::{Handle, EcsManager, resource::Resource};
use error::EngineError;

pub struct NullResourceLoader;

struct SomeResource;

impl Resource<NullResourceLoader> for SomeResource {
    type CreationData = ();

    fn create(
        _loader: &NullResourceLoader,
        _ecs: &EcsManager<NullResourceLoader>,
        _data: &()
    ) -> Result<Self, EngineError> {
        Ok(SomeResource)
    }

    fn release(&self, _loader: &NullResourceLoader) {}
}

#[test]
fn explicit_handles_can_read_back() {
    let mut ecs: EcsManager<NullResourceLoader> = EcsManager::new();
    let handle = Handle::for_resource(0x1);
    let resource = SomeResource;

    ecs.push_new_with_handle(handle, resource);
    let item_ref  = ecs.get_item::<SomeResource>(handle);
    assert!(item_ref.is_some());
}

#[test]
fn implicit_handles_count_logically() {
    let mut ecs: EcsManager<NullResourceLoader> = EcsManager::new();

    let handle_0  = ecs.add_item(SomeResource);
    ecs.add_item(SomeResource);
    ecs.add_item(SomeResource);
    let next_table_index  = ecs.next_index_guess::<SomeResource>().unwrap();
    assert_eq!(next_table_index, 3);

    ecs.remove_item::<SomeResource>(handle_0);
    let next_table_index  = ecs.next_index_guess::<SomeResource>().unwrap();
    assert_eq!(next_table_index, 0);

    ecs.add_item(SomeResource);
    let next_table_index  = ecs.next_index_guess::<SomeResource>().unwrap();
    assert_eq!(next_table_index, 1);

    ecs.add_item(SomeResource);
    let next_table_index  = ecs.next_index_guess::<SomeResource>().unwrap();
    assert_eq!(next_table_index, 4);
}

#[test]
fn implicit_handles_can_read_back() {
    let mut ecs: EcsManager<NullResourceLoader> = EcsManager::new();
    let handle_0  = ecs.add_item(SomeResource);
    ecs.add_item(SomeResource);
    let item_back  = ecs.remove_item::<SomeResource>(handle_0);
    assert!(item_back.is_some());
}

#[test]
fn unused_handles_read_back_as_none() {
    let mut ecs: EcsManager<NullResourceLoader> = EcsManager::new();
    ecs.add_item(SomeResource);
    ecs.add_item(SomeResource);
    let item_back  = ecs
        .remove_item::<SomeResource>(Handle::for_resource(5));
    assert!(item_back.is_none());
}
