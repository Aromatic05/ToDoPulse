use redb::Result;
use std::fs;
use std::hash::{Hash, Hasher};
use std::ops::DerefMut;
use tauri::Manager;
use tauri::State;

use crate::aigc::gen_tag;
use crate::data::{self, Event, EventMetadata, List, Tag};
use crate::storage::{Repository, StorageState};

#[tauri::command]
pub async fn add_event(
    state: State<'_, StorageState>,
    title: String,
    content: String,
    task_time: u64,
    tag: Option<Vec<String>>,
    app: State<'_, tauri::AppHandle>,
) -> Result<Event, String> {
    let metadata = EventMetadata::new();
    let content_path = app
        .path()
        .data_dir()
        .map_err(|e| e.to_string())?
        .join("events");
    if !content_path.exists() {
        fs::create_dir_all(&content_path).map_err(|e| e.to_string())?
    };
    let content_path = content_path.join(format!("{}.md", title));
    fs::write(&content_path, content).map_err(|e| e.to_string())?;
    let mut new_event = Event {
        metadata,
        title,
        content: content_path.to_string_lossy().to_string(),
        task_time,
        finished: false,
        priority: data::Priority::Undefined,
    };
    if let Some(tag_value) = tag {
        new_event.metadata.tag = Some(tag_value);
    } else {
        new_event.metadata.tag = gen_tag(state.clone(), &content_path)
            .await
            .map_err(|e| e.to_string())?;
    }
    let mut guard = state.0.lock().unwrap();
    let storage = guard.deref_mut();
    Repository::<Event>::add(storage, &new_event).map_err(|e| e.to_string())?;
    Ok(new_event.clone())
}

#[tauri::command]
pub async fn delete_event(state: State<'_, StorageState>, uuid: &str) -> Result<(), String> {
    let mut guard = state.0.lock().unwrap();
    let storage = guard.deref_mut();
    Repository::<Event>::delete(storage, uuid).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn new_list(
    state: State<'_, StorageState>,
    title: &str,
    icon: &str,
) -> Result<List, String> {
    let mut guard = state.0.lock().unwrap();
    let storage = guard.deref_mut();
    let new_list = data::List::new(title, icon);
    Repository::<data::List>::add(storage, &new_list).map_err(|e| e.to_string())?;
    Ok(new_list.clone())
}

#[tauri::command]
pub async fn delete_list(state: State<'_, StorageState>, title: String) -> Result<(), String> {
    let mut guard = state.0.lock().unwrap();
    let storage = guard.deref_mut();
    Repository::<data::List>::delete(storage, &title).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn get_lists(state: State<'_, StorageState>) -> Result<Vec<data::List>, String> {
    let mut guard = state.0.lock().unwrap();
    let storage = guard.deref_mut();
    let lists = Repository::<data::List>::get_all(storage).map_err(|e| e.to_string())?;
    Ok(lists)
}

fn tag_exists(state: &State<'_, StorageState>, name: &str) -> bool {
    let mut hash = std::collections::hash_map::DefaultHasher::new();
    name.hash(&mut hash);
    let hash = hash.finish();
    let mut guard = state.0.lock().unwrap();
    let storage = guard.deref_mut();
    let tag: Option<Tag> = Repository::<Tag>::get_by_name(storage, &hash.to_string())
        .map_err(|e| e.to_string())
        .unwrap();
    if tag.is_none() {
        return false;
    }
    true
}

#[tauri::command]
pub async fn add_tag(
    state: State<'_, StorageState>,
    tag: String,
    color: data::TagColor,
) -> Result<(), String> {
    if tag_exists(&state, &tag) {
        return Ok(());
    }
    let tag = Tag::new(tag, color);
    state.0.lock().unwrap().add(&tag).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_tags(state: State<'_, StorageState>) -> Result<Vec<Tag>, String> {
    let mut guard = state.0.lock().unwrap();
    let storage = guard.deref_mut();
    let tags = Repository::<Tag>::get_all(storage).map_err(|e| e.to_string())?;
    Ok(tags)
}

#[tauri::command]
pub async fn delete_tag(state: State<'_, StorageState>, tag: String) -> Result<(), String> {
    let mut guard = state.0.lock().unwrap();
    let storage = guard.deref_mut();
    Repository::<Tag>::delete(storage, &tag).map_err(|e| e.to_string())?;
    Ok(())
}
