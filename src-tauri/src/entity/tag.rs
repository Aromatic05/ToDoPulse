use redb::{self, TableDefinition};
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};
use std::ops::DerefMut;
use tauri::State;
use ts_rs::TS;

use crate::entity::{Repository, StorageState};
use crate::utils::tag_exists;
use crate::error::ErrorKind;

use super::Entity;

type Table = TableDefinition<'static, &'static [u8], &'static [u8]>;

const TAG_TABLE: Table = TableDefinition::new("tag");

#[derive(Serialize, Deserialize, TS, Clone)]
#[ts(export)]
pub enum TagColor {
    Primary,
    Secondary,
    Sucess,
    Info,
    Warning,
    Error,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Tag {
    id: u64,
    pub name: String,
    pub color: TagColor,
}

impl Entity for Tag {
    fn table_def() -> Table {
        TAG_TABLE
    }
    fn id_bytes(&self) -> Vec<u8> {
        self.id.to_string().into_bytes()
    }
    fn value(&self) -> Vec<u8> {
        serde_json::to_vec(self).unwrap()
    }
}

impl Tag {
    pub fn new(name: String, color: TagColor) -> Self {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        name.hash(&mut hasher);
        let id = hasher.finish();
        Self { id, name, color }
    }
}

#[tauri::command]
pub async fn add_tag(
    state: State<'_, StorageState>,
    tag: String,
    color: TagColor,
) -> Result<(), ErrorKind> {
    if tag_exists(&state, &tag).await {
        return Ok(());
    }
    let tag_obj = Tag::new(tag, color); // Renamed variable to avoid conflict
    let mut guard = state.0.lock().await;
    let storage = guard.deref_mut();
    Repository::<Tag>::add(storage, &tag_obj)?;
    Ok(())
}

#[tauri::command]
pub async fn get_tags(state: State<'_, StorageState>) -> Result<Vec<Tag>, ErrorKind> {
    let mut guard = state.0.lock().await;
    let storage = guard.deref_mut();
    let tags = Repository::<Tag>::get_all(storage)?;
    Ok(tags)
}

#[tauri::command]
pub async fn delete_tag(state: State<'_, StorageState>, tag: String) -> Result<(), ErrorKind> {
    let mut guard = state.0.lock().await;
    let storage = guard.deref_mut();
    Repository::<Tag>::delete(storage, &tag)?;
    Ok(())
}
