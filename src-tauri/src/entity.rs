pub mod event;
pub mod list;
pub mod tag;

use std::sync::Mutex;

use anyhow::{Ok, Result};
use redb::{self, Database, ReadableTable, TableDefinition};
use serde::{Deserialize, Serialize};
use tauri::Manager;

pub use event::{Event, FEvent};
pub use list::List;
pub use tag::{get_tags, Tag};

pub trait Entity: Clone + Serialize + for<'de> Deserialize<'de> {
    fn table_def() -> TableDefinition<'static, &'static [u8], &'static [u8]>;
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
    fn filter<F>(&self, filter_fn: F) -> Result<Vec<T>>
    where
        F: Fn(&T) -> bool;
    fn get_all(&self) -> Result<Vec<T>>;
}

pub struct StorageState(pub Mutex<Storage>);

#[allow(dead_code)]
pub struct Storage {
    db: Database,
}

impl Storage {
    pub fn new(app: &tauri::AppHandle) -> Result<Self> {
        let db = connect_to_db(app)?;
        Ok(Self { db })
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
            let key = name.parse::<u64>()?.to_le_bytes().to_vec();
            if let Some(value) = t.get(&key[..])? {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    use tempfile::tempdir;
    use std::ops::DerefMut;

    // 创建一个帮助函数来初始化测试环境
    fn setup_test_db() -> (StorageState, tempfile::TempDir) {
        // 创建临时目录用于测试数据库
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");

        // 创建测试数据库
        let db = redb::Database::create(db_path).unwrap();

        // 创建存储状态
        let storage = crate::entity::Storage { db };
        let state = StorageState(Mutex::new(storage));

        (state, temp_dir)
    }

    #[tokio::test]
    async fn test_get_by_name() {
        let (state, _temp_dir) = setup_test_db();
        let list = List::new("Test List", "icon.png");
        let mut guard = state.0.lock().unwrap();
        let storage = guard.deref_mut();
        Repository::<List>::add(storage, &list).unwrap();

        let result = Repository::<List>::get_by_name(storage, &list.id.to_string()).unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().title, "Test List");
    }
}
