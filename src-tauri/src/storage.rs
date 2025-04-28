use anyhow::{Ok, Result};
use redb::{self, Database, ReadableTable, TableDefinition};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::Manager;

type Table = TableDefinition<'static, &'static [u8], &'static [u8]>;

const LIST_TABLE: Table = TableDefinition::new("lists");
const EVENT_TABLE: Table = TableDefinition::new("events");
const TAG_TABLE: Table = TableDefinition::new("tag");

pub struct StorageState(pub Mutex<Storage>);

pub trait Entity: Serialize + for<'de> Deserialize<'de> {
    fn table_def() -> Table;
    fn id_bytes(&self) -> Vec<u8>;
    fn value(&self) -> Vec<u8>;
}

pub trait Repository<T: Entity> {
    fn add(&self, entity: &T) -> Result<()>;
    fn delete(&self, name: &str) -> Result<()>;
    #[allow(dead_code)]
    fn update<F>(&self, id: &str, update_fn: F) -> Result<()>
    where
        F: FnOnce(&mut T) -> Result<()>;
    fn get_by_name(&self, name: &str) -> Result<Option<T>>;
    fn get_all(&self) -> Result<Vec<T>>;
}

#[allow(dead_code)]
pub struct Storage {
    db: Database,
    event_repo: Table,
    list_repo: Table,
    tag_repo: Table,
}

impl Storage {
    pub fn new(app: &tauri::AppHandle) -> Result<Self> {
        let db = connect_to_db(app)?;
        let event_repo = EVENT_TABLE;
        let list_repo = LIST_TABLE;
        let tag_repo = TAG_TABLE;
        Ok(Self {
            db,
            event_repo,
            list_repo,
            tag_repo,
        })
    }
}

impl<T: Entity> Repository<T> for Storage {
    fn add(&self, entity: &T) -> Result<()> {
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
    fn delete(&self, name: &str) -> Result<()> {
        let txn = self.db.begin_write()?;
        let table = T::table_def();
        {
            let mut t = txn.open_table(table)?;
            let key = name.as_bytes();
            t.remove(key)?;
        }
        txn.commit()?;
        Ok(())
    }
    fn update<F>(&self, name: &str, update_fn: F) -> Result<()>
    where
        F: FnOnce(&mut T) -> Result<()>,
    {
        if let Some(mut entity) = self.get_by_name(name)? {
            update_fn(&mut entity)?;
            self.add(&entity)?;
            Ok(())
        } else {
            Err(anyhow::anyhow!("Entity not found"))
        }
    }
    fn get_by_name(&self, name: &str) -> Result<Option<T>> {
        let txn = self.db.begin_read()?;
        let table = T::table_def();
        {
            let t = txn.open_table(table)?;
            let key = name.as_bytes();
            if let Some(value) = t.get(key)? {
                let name = serde_json::from_slice(value.value())?;
                return Ok(Some(name));
            }
        }
        Ok(None)
    }
    fn get_all(&self) -> Result<Vec<T>> {
        let txn = self.db.begin_read()?;
        let table = T::table_def();
        {
            let t = txn.open_table(table)?;
            let mut result = Vec::new();
            for entry in t.iter()? {
                let (_, value) = entry?;
                let entity: T = serde_json::from_slice(value.value())?;
                result.push(entity);
            }
            return Ok(result);
        }
    }
}

fn connect_to_db(app: &tauri::AppHandle) -> Result<Database> {
    let data_dir = app.path().data_dir()?.join("events");
    if !data_dir.exists() {
        std::fs::create_dir_all(&data_dir)?;
    }
    let db_path = data_dir.join("events.db");
    if !db_path.exists() {
        std::fs::create_dir_all(&data_dir)?;
    }
    let db = Database::create(db_path)?;
    Ok(db)
}
