use tauri::State;
use core::hash;
use std::hash::{Hash, Hasher};

use crate::storage::{StorageState,Repository};
use crate::data::{self, Event, EventMetadata, EventType, List, Tag, TaskTime};

#[tauri::command]
pub fn get_metadata(event: Event) -> EventMetadata {
    event.metadata.clone()
}

#[tauri::command]
pub fn new_event(
    title: String,
    content: String,
    event_type: String,
    task_time: TaskTime,
) -> Event {
    let metadata = EventMetadata::new();
    let event_type = match event_type.as_str() {
        "Instant" => EventType::Instant,
        "Duration" => EventType::Duration,
        _ => panic!("Invalid event type"),
    };
    Event {
        metadata,
        title,
        content,
        event_type,
        task_time,
        finished: false,
    }
}

#[tauri::command]
pub async fn add_event(state: State<'_, StorageState>, event: Event) -> Result<(), String> {
    state
        .0
        .lock().unwrap()
        .add(event)
        .map_err(|e| e.to_string())
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

fn tag_exists(state: &State<'_, StorageState>,name: &str) -> bool {
    let mut hash = std::collections::hash_map::DefaultHasher::new();
    name.hash(&mut hash);
    let hash = hash.finish();
    let tag: Option<Tag> = state
        .0
        .lock().unwrap()
        .get_by_name(&hash.to_string()).unwrap();
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
    state
        .0
        .lock().unwrap()
        .add(tag)
        .map_err(|e| e.to_string())
}
