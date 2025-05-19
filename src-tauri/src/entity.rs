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

pub struct StorageState<R = tauri::Wry>(pub Mutex<Storage>, pub Mutex<App<R>>)
where
    R: tauri_runtime::Runtime<tauri::EventLoopMessage>;

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
    R: tauri_runtime::Runtime<tauri::EventLoopMessage>,
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
    Ok(db)
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::ops::DerefMut;
  use tokio::sync::Mutex;
  use std::time::Instant;
  use tauri::test::{mock_app, MockRuntime};
  use tauri::Manager;
  use tempfile::tempdir;

  // 创建一个帮助函数来初始化测试环境
  fn setup() -> (StorageState<MockRuntime>, tempfile::TempDir) {
    // 创建临时目录用于测试数据库
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");

    // 创建测试数据库
    let db = redb::Database::create(db_path).unwrap();
    let app = mock_app();
    let app_handle = app.app_handle();
    // 创建存储状态
    let storage = crate::entity::Storage { db };
    let state = StorageState::<MockRuntime>(
    Mutex::new(storage),
    Mutex::new(App::<MockRuntime>::new(&app_handle)),
    );
    (state, temp_dir)
  }

  #[tokio::test]
  async fn setup_test() {
    let start = Instant::now();
    let (_state, _temp_dir) = setup();
    
    println!("setup_test completed in: {:?}", start.elapsed());
  }


  #[tokio::test]
  async fn test_get_by_name() {
    let start = Instant::now();
    let (state, _temp_dir) = setup();
    let mut guard = state.0.lock().await;
    let storage = guard.deref_mut();

    let list = List::new("Test List", "icon.png");
    Repository::<List>::add(storage, &list).unwrap();
    let result = Repository::<List>::get_by_name(storage, &list.uuid.to_string()).unwrap();
    assert!(result.is_some());
    assert_eq!(result.unwrap().title, "Test List");

    let event = Event::new("Test Event", "Test Content");
    Repository::<Event>::add(storage, &event).unwrap();
    let result =
      Repository::<Event>::get_by_name(storage, &event.metadata.uuid.to_string()).unwrap();
    assert!(result.is_some());
    assert_eq!(result.unwrap().title, "Test Event");
    
    println!("test_get_by_name completed in: {:?}", start.elapsed());
  }

  #[tokio::test]
  async fn test_filter() {
    let start = Instant::now();
    let (state, _temp_dir) = setup();
    let list1 = List::new("Test List 1", "icon1.png");
    let list2 = List::new("Test List 2", "icon2.png");
    let mut guard = state.0.lock().await;
    let storage = guard.deref_mut();
    Repository::<List>::add(storage, &list1).unwrap();
    Repository::<List>::add(storage, &list2).unwrap();

    let result = Repository::<List>::filter(storage, |l| l.title.contains("1")).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].title, "Test List 1");
    
    println!("test_filter completed in: {:?}", start.elapsed());
  }
  
  #[tokio::test]
  async fn test_delete() {
    let start = Instant::now();
    let (state, _temp_dir) = setup();
    let mut guard = state.0.lock().await;
    let storage = guard.deref_mut();
    
    let list = List::new("Test Delete", "icon.png");
    let list_id = list.uuid.to_string();
    
    Repository::<List>::add(storage, &list).unwrap();
    assert!(Repository::<List>::get_by_name(storage, &list_id).unwrap().is_some());
    
    Repository::<List>::delete(storage, &list_id).unwrap();
    assert!(Repository::<List>::get_by_name(storage, &list_id).unwrap().is_none());
    
    println!("test_delete completed in: {:?}", start.elapsed());
  }
  
  #[tokio::test]
  async fn test_ensure_table_exists() {
    let start = Instant::now();
    let (state, _temp_dir) = setup();
    let mut guard = state.0.lock().await;
    let storage = guard.deref_mut();
    
    
    // After ensuring the table exists, we should be able to add items
    let list = List::new("Test Table", "icon.png");
    Repository::<List>::add(storage, &list).unwrap();
    
    println!("test_ensure_table_exists completed in: {:?}", start.elapsed());
  }
  
  #[tokio::test]
  async fn test_update() {
    let start = Instant::now();
    let (state, _temp_dir) = setup();
    let mut guard = state.0.lock().await;
    let storage = guard.deref_mut();
    
    let list = List::new("Original Title", "icon.png");
    let list_id = list.uuid.to_string();
    
    Repository::<List>::add(storage, &list).unwrap();
    
    Repository::<List>::update(storage, &list_id, |l| {
      l.title = "Updated Title".to_string();
      Ok(())
    }).unwrap();
    
    let updated = Repository::<List>::get_by_name(storage, &list_id).unwrap();
    assert!(updated.is_some());
    assert_eq!(updated.unwrap().title, "Updated Title");
    
    println!("test_update completed in: {:?}", start.elapsed());
  }
  
  #[tokio::test]
  async fn test_get_all() {
    const MAX_LISTS: usize = 10000;
    let start = Instant::now();
    let (state, _temp_dir) = setup();
    let mut guard = state.0.lock().await;
    let storage = guard.deref_mut();
    
    // Add multiple items
    for i in 1..=MAX_LISTS {
      let list = List::new(&format!("List {}", i), "icon.png");
      Repository::<List>::add(storage, &list).unwrap();
    }
    println!("Added {} lists in: {:?}", MAX_LISTS, start.elapsed());
    let all_lists = Repository::<List>::get_all(storage).unwrap();
    assert_eq!(all_lists.len(), MAX_LISTS);
    
    println!("test_get_all completed in: {:?}", start.elapsed());
  }
  
  #[tokio::test]
  async fn test_performance_bulk_operations() {
    let start = Instant::now();
    let (state, _temp_dir) = setup();
    let mut guard = state.0.lock().await;
    let storage = guard.deref_mut();
    
    // Test bulk insert performance
    let insert_start = Instant::now();
    for i in 1..=100 {
      let list = List::new(&format!("Bulk List {}", i), "icon.png");
      Repository::<List>::add(storage, &list).unwrap();
    }
    let insert_time = insert_start.elapsed();
    println!("Bulk insert (100 items) completed in: {:?}", insert_time);
    
    // Test bulk read performance
    let read_start = Instant::now();
    let all_lists = Repository::<List>::get_all(storage).unwrap();
    let read_time = read_start.elapsed();
    assert_eq!(all_lists.len(), 100);
    println!("Bulk read (100 items) completed in: {:?}", read_time);
    
    // Test bulk filter performance
    let filter_start = Instant::now();
    let filtered = Repository::<List>::filter(storage, |l| l.title.contains("5")).unwrap();
    let filter_time = filter_start.elapsed();
    println!("Bulk filter (found {} items) completed in: {:?}", filtered.len(), filter_time);
    
    println!("test_performance_bulk_operations total time: {:?}", start.elapsed());
  }
}
