use anyhow::Result;
use chrono::Utc;
use redb::{self, TableDefinition};
use serde::{Deserialize, Serialize};
use std::fs;
use std::ops::DerefMut;
use tauri::State;
use ts_rs::TS;
use uuid::Uuid;

use super::{Entity, Repository, StorageState};
use crate::filter::map_filter;
use crate::function::gen_tag;
use crate::utils::{event_to_fevent, AppPaths};

type Table = TableDefinition<'static, &'static [u8], &'static [u8]>;

const EVENT_TABLE: Table = TableDefinition::new("events");

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct EventMetadata {
    pub uuid: String,
    pub timestamp: u64,
    pub list: Option<String>,
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

#[cfg(test)]
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
    pub listid: String,
    pub tag: Option<Vec<String>>,
    pub title: String,
    pub create: String,
    pub ddl: String,
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
    metadata.list = listid.map(|id| id.to_string());  // 直接使用字符串，不再解析为u64
    let content_path = AppPaths::data_dir().join(format!("{}.md", title));
    fs::write(&content_path, "").map_err(|e| e.to_string())?;
    let mut new_event = Event {
        metadata,
        title: title.to_string(),
        content: content_path.to_string_lossy().to_string(),
        task_time: if ddl.is_empty() {
            None
        } else {
            ddl.parse::<u64>().ok()
        },
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
pub async fn update_event(state: State<'_, StorageState>, f_event: FEvent) -> Result<(), String> {
    let mut guard = state.0.lock().unwrap();
    let storage = guard.deref_mut();
    let old_event =
        Repository::<Event>::get_by_name(storage, &f_event.id).map_err(|e| e.to_string())?;
    if let Some(mut new) = old_event {
        new.metadata.tag = f_event.tag;
        new.title = f_event.title;
        new.task_time = f_event.ddl.parse::<u64>().ok();
        new.finished = f_event.finished;
        new.priority = f_event.priority;
        new.color = f_event.color;
        new.icon = f_event.icon;
        new.metadata.list = Some(f_event.listid);  // 直接使用字符串类型的listid
        Repository::<Event>::add(storage, &new).map_err(|e| e.to_string())?;
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

#[tauri::command]
pub async fn filter_events(
    state: State<'_, StorageState>,
    filter: &str,
) -> Result<Vec<FEvent>, String> {
    let mut guard = state.0.lock().unwrap();
    let storage = guard.deref_mut();
    let res = Repository::<Event>::filter(storage, map_filter(filter).unwrap());
    match res {
        Ok(events) => Ok(events
            .into_iter()
            .map(|event| event_to_fevent(&event))
            .collect()),
        Err(e) => {
            return Err(e.to_string());
        }
    }
}
