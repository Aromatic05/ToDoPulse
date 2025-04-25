use anyhow::{Ok, Result};
use chrono::Utc;
use rand;
use redb::{self, Database, ReadableTable, TableDefinition};
use serde::{Deserialize, Serialize};
use std::{sync::Mutex, vec};
use tauri::Manager;
use uuid::Uuid;

use crate::filter;

type Table = TableDefinition<'static, &'static [u8], &'static [u8]>;

const LIST_TABLE: Table = TableDefinition::new("lists");
const EVENT_TABLE: Table = TableDefinition::new("events");

pub struct StorageState(pub Mutex<Storage>);

trait Entity: Serialize + for<'de> Deserialize<'de> {
    fn table_def() -> Table;
    fn id_bytes(&self) -> Vec<u8>;
    fn value(&self) -> Vec<u8>;
}

pub trait Repository<T: Entity> {
    fn add(&self, entity: T) -> Result<()>;
    fn delete(&self, id: &[u8]) -> Result<()>;
    fn get_all(&self) -> Result<Vec<T>>;
    fn get_by_id(&self, id: &[u8]) -> Result<Option<T>>;
}

pub struct Storage {
    db: Database,
    event_repo: Table,
    list_repo: Table,
}

impl Storage {
    pub fn new(app: &tauri::AppHandle) -> Result<Self> {
        let db = connect_to_db(app)?;
        let event_repo = EVENT_TABLE;
        let list_repo = LIST_TABLE;
        Ok(Self {
            db,
            event_repo,
            list_repo,
        })
    }
}

impl<T: Entity> Repository<T> for Storage {
    fn add(&self, entity: T) -> Result<()> {
        let txn = self.db.begin_write()?;
        let table = T::table_def();
        {
            let mut t = txn.open_table(table)?;
            let key = entity.id_bytes();
            let value = entity.value();
            t.insert(&key[..], &value[..])?;
        }
        txn.commit()?;
        Ok(())
    }
    fn delete(&self, id: &[u8]) -> Result<()> {
        let txn = self.db.begin_write()?;
        let table = T::table_def();
        {
            let mut t = txn.open_table(table)?;
            let key = id;
            t.remove(key)?;
        }
        txn.commit()?;
        Ok(())
    }
    fn get_all(&self) -> Result<Vec<T>> {
        todo!()
    }
    fn get_by_id(&self, id: &[u8]) -> Result<Option<T>> {
        todo!()
    }
}


fn connect_to_db(app: &tauri::AppHandle) -> Result<Database> {
    let data_dir = app.path().data_dir()?.join("events");
    let db_path = data_dir.join("events.db");
    if !db_path.exists() {
        std::fs::create_dir_all(&data_dir)?;
    }
    let db = Database::create(db_path)?;
    Ok(db)
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct EventMetadata {
    uuid: String,
    timestamp: u64,
}

impl EventMetadata {
    pub fn new() -> Self {
        Self {
            uuid: Uuid::new_v4().to_string(),
            timestamp: Utc::now().timestamp_millis() as u64,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum EventType {
    Instant,
    Duration,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct DurationTime {
    pub start: u64,
    pub end: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum TaskTime {
    Deadline(u64),
    Duration(DurationTime),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
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

#[derive(Serialize, Deserialize, Clone)]
pub struct List {
    pub id: u8,
    pub name: String,
}

impl Entity for List {
    fn table_def() -> Table {
        LIST_TABLE
    }
    fn id_bytes(&self) -> Vec<u8> {
        vec![self.id]
    }
    fn value(&self) -> Vec<u8> {
        serde_json::to_vec(self).unwrap()
    }
}

impl List {
    pub fn new(name: &str) -> Self {
        Self {
            id: rand::random::<u8>(),
            name: name.to_string(),
        }
    }
}
