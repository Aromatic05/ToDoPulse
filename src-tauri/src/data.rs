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

#[derive(Serialize, Deserialize,Clone, PartialEq)]
pub struct EventMetadata {
    pub uuid: String,
    pub timestamp: u64,
    pub list: Option<u8>,
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
    pub task_time: u64,
    pub finished: bool,
    pub priority: Priority,
    pub icon: String,
    pub color: String,
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

#[derive(Serialize, Deserialize, TS, Clone)]
#[ts(export)]
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
        self.title.as_bytes().to_vec()
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
