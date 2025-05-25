use redb::{self, TableDefinition};
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};
use std::ops::DerefMut;
use tauri::State;
use ts_rs::TS;

use crate::entity::{Repository, StorageState};
use crate::error::ErrorKind;
use crate::utils::tag_exists;

use super::Entity;
use super::{Event, FEvent};

type Table = TableDefinition<'static, &'static [u8], &'static [u8]>;

const TAG_TABLE: Table = TableDefinition::new("tag");

#[derive(Serialize, Deserialize, TS, Clone)]
#[ts(export)]
pub enum TagColor {
    Primary,
    Secondary,
    Success,
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
        serde_json::to_vec(self).unwrap_or_default()
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

/// Creates a new tag in the database
///
/// Creates a tag with the given name and color if it doesn't already exist.
/// The tag's ID is generated from a hash of the tag name.
///
/// # Parameters
/// * `state` - Application state containing the database connection
/// * `tag` - Name of the new tag
/// * `color` - Color category for the new tag from the TagColor enum
///
/// # Returns
/// * `Result<(), ErrorKind>` - Success or an error if the tag couldn't be created
/// * Returns success without an error if the tag already exists
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

/// Retrieves all tags from the database
///
/// Fetches all tags from the database and returns them.
///
/// # Parameters
/// * `state` - Application state containing the database connection
///
/// # Returns
/// * `Result<Vec<Tag>, ErrorKind>` - List of all tags or an error
#[tauri::command]
pub async fn get_tags(state: State<'_, StorageState>) -> Result<Vec<Tag>, ErrorKind> {
    let mut guard = state.0.lock().await;
    let storage = guard.deref_mut();
    let tags = Repository::<Tag>::get_all(storage)?;
    Ok(tags)
}

/// Deletes a tag from the database
///
/// Removes the specified tag from the database.
///
/// # Parameters
/// * `state` - Application state containing the database connection
/// * `tag` - Name of the tag to delete
///
/// # Returns
/// * `Result<(), ErrorKind>` - Success or an error if the tag couldn't be deleted
#[tauri::command]
pub async fn delete_tag(state: State<'_, StorageState>, tag: &str) -> Result<(), ErrorKind> {
    let mut guard = state.0.lock().await;
    let storage = guard.deref_mut();
    Repository::<Tag>::delete(storage, &tag)?;
    Ok(())
}

/// Get the events with the given tag
///
/// Retrieves all events that have been tagged with the specified tag.
///
/// # Parameters
/// * `state` - Application state containing the database connection
/// * `tag` - Name of the tag to filter events by
///
/// # Returns
/// * `Result<Vec<FEvent>, ErrorKind>` - List of matching events in frontend format or an error
#[tauri::command]
pub async fn tag_content(
    state: State<'_, StorageState>,
    tag: &str,
) -> Result<Vec<FEvent>, ErrorKind> {
    let mut guard = state.0.lock().await;
    let storage = guard.deref_mut();
    let events = Repository::<Event>::filter(storage, |event| {
        event
            .metadata
            .tag
            .as_ref()
            .map_or(false, |tags| tags.iter().any(|t| t == tag))
    })?;
    let f_events: Vec<FEvent> = events
        .into_iter()
        .map(|event| FEvent::from(event))
        .collect();
    Ok(f_events)
}
