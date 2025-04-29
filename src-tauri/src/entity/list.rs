use redb::{self, TableDefinition};
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};
use std::ops::DerefMut;
use tauri::State;
use ts_rs::TS;

use crate::entity::{Event, FEvent, Repository, StorageState};
use crate::utils::{event_to_fevent, list_exists};

use super::Entity;

type Table = TableDefinition<'static, &'static [u8], &'static [u8]>;

const LIST_TABLE: Table = TableDefinition::new("lists");

#[derive(Serialize, Deserialize, TS, Clone)]
#[ts(export)]
pub struct FList {
    pub id: String,
    pub title: String,
    pub icon: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct List {
    pub id: u64,
    pub title: String,
    pub icon: String,
}

impl Entity for List {
    fn table_def() -> Table {
        LIST_TABLE
    }
    fn id_bytes(&self) -> Vec<u8> {
        self.id.to_le_bytes().to_vec()
    }
    fn value(&self) -> Vec<u8> {
        serde_json::to_vec(self).unwrap()
    }
}

impl List {
    pub fn new(title: &str, icon: &str) -> Self {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        title.hash(&mut hasher);
        let id = hasher.finish();
        let icon = icon.to_string();
        let title = title.to_string();
        Self { title, id, icon }
    }
}

#[tauri::command]
pub async fn new_list(
    state: State<'_, StorageState>,
    title: &str,
    icon: &str,
) -> Result<List, String> {
    let mut guard = state.0.lock().unwrap();
    let storage = guard.deref_mut();
    let new_list = List::new(title, icon);
    Repository::<List>::add(storage, &new_list).map_err(|e| e.to_string())?;
    Ok(new_list.clone())
}

#[tauri::command]
pub async fn delete_list(state: State<'_, StorageState>, title: String) -> Result<(), String> {
    let mut guard = state.0.lock().unwrap();
    let storage = guard.deref_mut();
    Repository::<List>::delete(storage, &title).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn get_lists(state: State<'_, StorageState>) -> Result<Vec<FList>, String> {
    let mut guard = state.0.lock().unwrap();
    let storage = guard.deref_mut();
    let lists = Repository::<List>::get_all(storage).map_err(|e| e.to_string())?;
    let f_lists: Vec<FList> = lists
        .into_iter()
        .map(|list| FList {
            id: list.id.to_string(),
            title: list.title,
            icon: list.icon,
        })
        .collect();
    Ok(f_lists)
}

#[tauri::command]
pub async fn list_content(
    state: State<'_, StorageState>,
    listid: &str,
) -> Result<Vec<FEvent>, String> {
    if !list_exists(&state, listid) {
        Err("List not found".to_string())
    } else {
        let mut guard = state.0.lock().unwrap();
        let storage = guard.deref_mut();
        let evnets = Repository::<Event>::filter(storage, |event| {
            if let Some(list) = event.metadata.list {
                list.to_string() == listid
            } else {
                false
            }
        })
        .map_err(|e| e.to_string())?;
        let f_events: Vec<FEvent> = evnets
            .into_iter()
            .map(|event| event_to_fevent(&event))
            .collect();
        Ok(f_events)
    }
}

#[tauri::command]
pub async fn rename_list(
    state: State<'_, StorageState>,
    listid: &str,
    new: &str,
) -> Result<(), String> {
    if !list_exists(&state, listid) {
        return Err("List not found".to_string());
    }
    let mut guard = state.0.lock().unwrap();
    let storage = guard.deref_mut();
    Repository::<List>::update(storage, new, |list| {
        list.title = new.to_string();
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        new.hash(&mut hasher);
        list.id = hasher.finish();
        Ok(())
    })
    .map_err(|e| e.to_string())?;
    Ok(())
}
