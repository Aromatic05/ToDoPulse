use redb::Result;
use std::hash::{Hash, Hasher};
use std::ops::DerefMut;
use tauri::State;

use crate::aigc::gen_tag;
use crate::data::{self, Event, EventMetadata, EventType, Tag, TaskTime};
use crate::storage::{Repository,StorageState};

#[tauri::command]
pub fn get_metadata(event: Event) -> EventMetadata {
    event.metadata.clone()
}

#[tauri::command]
pub async fn new_event(
    state: State<'_, StorageState>,
    title: String,
    content: String,
    event_type: String,
    task_time: TaskTime,
) -> Result<Event, String> {
    let metadata = EventMetadata::new();
    let event_type = match event_type.as_str() {
        "Instant" => EventType::Instant,
        "Duration" => EventType::Duration,
        _ => panic!("Invalid event type"),
    };
    let mut new_evnet = Event {
        metadata,
        title,
        content,
        event_type,
        task_time,
        finished: false,
    };
    gen_tag(state, &mut new_evnet)
        .await
        .map_err(|e| e.to_string())?;
    Ok(new_evnet)
}

#[tauri::command]
pub async fn add_event(state: State<'_, StorageState>, event: Event) -> Result<(), String> {
  let mut guard = state.0.lock().unwrap();
  let storage = guard.deref_mut();
  Repository::<Event>::add(storage, event)
      .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn delete_event(state: State<'_, StorageState>, uuid: String) -> Result<(), String> {
    todo!()
}

#[tauri::command]
pub async fn get_events(
    state: State<'_, StorageState>,
    deadline: u64,
) -> Result<Vec<Event>, String> {
    todo!()
}

#[tauri::command]
pub async fn new_list(state: State<'_, StorageState>, name: &str) -> Result<(), String> {
    todo!()
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
    state.0.lock().unwrap().add(tag).map_err(|e| e.to_string())
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
