use chrono::Utc;
use redb::{self, TableDefinition};
use serde::{Deserialize, Serialize};
use std::fs;
use std::ops::DerefMut;
use tauri::State;
use ts_rs::TS;
use uuid::Uuid;

use crate::aigc::gen_tag;
use crate::entity::{Repository, StorageState};
use crate::path::AppPaths;

use super::Entity;

type Table = TableDefinition<'static, &'static [u8], &'static [u8]>;

const EVENT_TABLE: Table = TableDefinition::new("events");

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct EventMetadata {
    pub uuid: String,
    pub timestamp: u64,
    pub list: Option<u64>,
    pub tag: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Time {
    pub date: String,
    pub time: String,
}

impl EventMetadata {
    pub fn new() -> Self {
        Self {
            uuid: Uuid::new_v4().to_string(),
            timestamp: Utc::now().timestamp_millis() as u64,
            tag: None,
            list: None,
        }
    }
}

#[derive(Serialize, Deserialize, TS, Clone)]
pub enum Priority {
    Low,
    Medium,
    High,
    Undefined,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Event {
    pub metadata: EventMetadata,
    pub title: String,
    pub content: String,
    pub task_time: Option<u64>,
    pub finished: bool,
    pub priority: Priority,
    pub icon: String,
    pub color: String,
}

impl Event {
    pub fn new(title: &str, content: &str) -> Self {
        let metadata = EventMetadata::new();
        Self {
            metadata,
            title: title.to_string(),
            content: content.to_string(),
            task_time: None,
            finished: false,
            priority: Priority::Undefined,
            icon: "default".to_string(),
            color: "default".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, TS, Clone)]
#[ts(export)]
pub struct FEvent {
    pub id: String,
    pub time: String,
    pub date: String,
    pub listid: String,
    pub tag: Option<Vec<String>>,
    pub title: String,
    pub create: String,
    pub finished: bool,
    pub priority: Priority,
    pub icon: String,
    pub color: String,
}

impl Entity for Event {
    fn table_def() -> Table {
        EVENT_TABLE
    }
    fn id_bytes(&self) -> Vec<u8> {
        self.metadata.uuid.as_bytes().to_vec()
    }
    fn value(&self) -> Vec<u8> {
        serde_json::to_vec(self).unwrap()
    }
}

#[tauri::command]
pub async fn add_event(
    state: State<'_, StorageState>,
    title: &str,
    listid: Option<&str>,
    priority: Priority,
    ddl: &str,
) -> Result<Event, String> {
    let mut metadata = EventMetadata::new();
    metadata.list = match listid {
        Some(id) => Some(id.parse::<u64>().map_err(|e| e.to_string())?),
        None => None,
    };
    let content_path = AppPaths::data_dir().join(format!("{}.md", title));
    let mut new_event = Event {
        metadata,
        title: title.to_string(),
        content: content_path.to_string_lossy().to_string(),
        task_time: if ddl.is_empty() {None} else {ddl.parse::<u64>().ok()},
        finished: false,
        priority,
        color: "default".to_string(),
        icon: "default".to_string(),
    };
    new_event.metadata.tag = gen_tag(state.clone(), &content_path)
        .await
        .map_err(|e| e.to_string())?;
    let mut guard = state.0.lock().unwrap();
    let storage = guard.deref_mut();
    Repository::<Event>::add(storage, &new_event).map_err(|e| e.to_string())?;
    Ok(new_event.clone())
}

#[tauri::command]
pub async fn event_content(state: State<'_, StorageState>, uuid: &str) -> Result<String, String> {
    let mut guard = state.0.lock().unwrap();
    let storage = guard.deref_mut();
    let event = Repository::<Event>::get_by_name(storage, uuid).map_err(|e| e.to_string())?;
    if let Some(event) = event {
        let content = fs::read_to_string(&event.content).map_err(|e| e.to_string())?;
        return Ok(content);
    }
    Ok("".to_string())
}

#[tauri::command]
pub async fn write_content(
    state: State<'_, StorageState>,
    uuid: &str,
    content: String,
) -> Result<(), String> {
    let mut guard = state.0.lock().unwrap();
    let storage = guard.deref_mut();
    let event = Repository::<Event>::get_by_name(storage, uuid).map_err(|e| e.to_string())?;
    if let Some(event) = event {
        fs::write(&event.content, content).map_err(|e| e.to_string())?;
        return Ok(());
    }
    Err("Event not found".to_string())
}

#[tauri::command]
pub async fn put_event(state: State<'_, StorageState>, event: FEvent) -> Result<(), String> {
    let mut guard = state.0.lock().unwrap();
    let storage = guard.deref_mut();
    let event = Repository::<Event>::get_by_name(storage, &event.id).map_err(|e| e.to_string())?;
    if let Some(mut event) = event {
        event.title = event.title;
        event.finished = event.finished;
        event.priority = event.priority;
        event.task_time = event.task_time;
        Repository::<Event>::add(storage, &event).map_err(|e| e.to_string())?;
        return Ok(());
    }
    Err("Event not found".to_string())
}

#[tauri::command]
pub async fn delete_event(state: State<'_, StorageState>, uuid: &str) -> Result<(), String> {
    let mut guard = state.0.lock().unwrap();
    let storage = guard.deref_mut();
    Repository::<Event>::delete(storage, uuid).map_err(|e| e.to_string())?;
    Ok(())
}
