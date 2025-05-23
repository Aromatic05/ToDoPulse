use std::ops::DerefMut;

use tauri::State;

use crate::entity::{
    Entity,
    List,
    Repository,
    StorageState, 
    Tag,          
};

async fn exists<T>(state: &State<'_, StorageState>, id: &str) -> bool
where
    T: Entity + 'static,
{
    let mut guard = state.0.lock().await;
    let storage = guard.deref_mut();

    let item: Option<T> = Repository::<T>::get_by_name(storage, &id.to_string())
        .map_err(|e| e.to_string())
        .unwrap();

    item.is_some()
}

pub async fn tag_exists(state: &State<'_, StorageState>, name: &str) -> bool {
    exists::<Tag>(state, name).await
}

pub async fn list_exists(state: &State<'_, StorageState>, uuid: &str) -> bool {
    exists::<List>(state, uuid).await
}
