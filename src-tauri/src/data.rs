use chrono::Utc;
use redb::{self, TableDefinition};
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};
use ts_rs::TS;
use uuid::Uuid;

use crate::storage::Entity;

type Table = TableDefinition<'static, &'static [u8], &'static [u8]>;

const LIST_TABLE: Table = TableDefinition::new("lists");
const EVENT_TABLE: Table = TableDefinition::new("events");
const TAG_TABLE: Table = TableDefinition::new("tag");

#[derive(Serialize, Deserialize, TS, Clone, PartialEq)]
pub struct EventMetadata {
    uuid: String,
    timestamp: u64,
    list: Option<u8>,
    pub tag: Option<Vec<String>>,
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

#[derive(Serialize, Deserialize, TS, Clone, PartialEq)]
pub enum EventType {
    Instant,
    Duration,
}

#[derive(Serialize, Deserialize, TS, Clone, PartialEq)]
pub struct DurationTime {
    pub start: u64,
    pub end: u64,
}

#[derive(Serialize, Deserialize, TS, Clone, PartialEq)]
pub enum TaskTime {
    Deadline(u64),
    Duration(DurationTime),
}

#[derive(Serialize, Deserialize, TS, Clone)]
#[ts(export)]
pub struct Event {
    pub metadata: EventMetadata,
    pub title: String,
    pub content: String,
    pub event_type: EventType,
    pub task_time: TaskTime,
    pub finished: bool,
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

#[derive(Serialize, Deserialize, TS, Clone)]
#[ts(export)]
pub struct List {
    id: u64,
    pub name: String,
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
        self.name.as_bytes().to_vec()
    }
}

impl List {
    pub fn new(name: &str, icon: &str) -> Self {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        name.hash(&mut hasher);
        let id = hasher.finish();
        let icon = icon.to_string();
        let name = name.to_string();
        Self { name, id, icon }
    }
}

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
        self.id.to_le_bytes().to_vec()
    }
    fn value(&self) -> Vec<u8> {
        self.name.as_bytes().to_vec()
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
