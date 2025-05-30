pub mod event;
pub mod list;
pub mod tag;

use tokio::sync::Mutex;

use anyhow::{Ok, Result};
use redb::{self, Database, ReadableTable, TableDefinition};
use serde::{Deserialize, Serialize};

use crate::utils::path::AppPaths;

pub use event::{Event, FEvent};
pub use list::List;
pub use tag::{get_tags, Tag};

const DB_NAME: &str = "data.db";
type Table = TableDefinition<'static, &'static [u8], &'static [u8]>;
const EVENT_TABLE: Table = TableDefinition::new("events");
const LIST_TABLE: Table = TableDefinition::new("lists");
const TAG_TABLE: Table = TableDefinition::new("tag");

pub trait Entity: Clone + Serialize + for<'de> Deserialize<'de> {
    fn table_def() -> TableDefinition<'static, &'static [u8], &'static [u8]>;
    fn id_bytes(&self) -> Vec<u8>;
    fn value(&self) -> Vec<u8>;
}

pub trait Repository<T: Entity> {
    fn add(&self, entity: &T) -> Result<()>;
    fn delete(&self, name: &str) -> Result<()>;
    fn update<F>(&self, id: &str, update_fn: F) -> Result<()>
    where
        F: FnOnce(&mut T) -> Result<()>;
    fn get_by_name(&self, name: &str) -> Result<Option<T>>;
    fn filter<F>(&self, filter_fn: F) -> Result<Vec<T>>
    where
        F: Fn(&T) -> bool;
    fn get_all(&self) -> Result<Vec<T>>;
}

pub struct StorageState<R = tauri::Wry>(pub Mutex<Storage>, pub Mutex<App<R>>)
where
    R: tauri::Runtime;

pub struct Storage {
    db: Database,
}

impl Storage {
    pub fn new() -> Result<Self> {
        let db = connect_to_db()?;
        Ok(Self { db })
    }
}

pub struct App<R = tauri::Wry>
where
    R: tauri::Runtime,
{
    app: tauri::AppHandle<R>,
}

impl<R: tauri::Runtime> App<R> {
    pub fn new(app: &tauri::AppHandle<R>) -> Self {
        Self { app: app.clone() }
    }
    #[allow(dead_code)]
    pub fn handle(&self) -> &tauri::AppHandle<R> {
        &self.app
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

    fn filter<F>(&self, filter_fn: F) -> Result<Vec<T>>
    where
        F: Fn(&T) -> bool,
    {
        let txn = self.db.begin_read()?;
        let storage = txn.open_table(T::table_def())?;
        let mut result = Vec::new();
        for entry in storage.iter()? {
            let (_, value) = entry?;
            let entity: T = serde_json::from_slice(value.value())?;
            if filter_fn(&entity) {
                result.push(entity);
            }
        }
        Ok(result)
    }

    fn get_by_name(&self, name: &str) -> Result<Option<T>> {
        let txn = self.db.begin_read()?;
        let table = T::table_def();
        {
            let t = txn.open_table(table)?;
            let key = name.as_bytes().to_vec();
            if let Some(value) = t.get(&key[..])? {
                let name = serde_json::from_slice(value.value())?;
                return Ok(Some(name));
            }
        }
        Ok(None)
    }

    fn get_all(&self) -> Result<Vec<T>> {
        self.filter(|_| true)
    }
}

fn connect_to_db() -> Result<Database> {
    let db_path = AppPaths::data_dir().join(DB_NAME);
    let db = Database::create(db_path)?;
    let txn = db.begin_write()?;
    {
        let _ = txn.open_table(EVENT_TABLE)?;
        let _ = txn.open_table(LIST_TABLE)?;
        let _ = txn.open_table(TAG_TABLE)?;
    }
    txn.commit()?;
    Ok(db)
}
