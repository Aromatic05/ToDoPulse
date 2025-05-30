use crate::entity::*;
use std::ops::DerefMut;
use std::time::Instant;
use tauri::test::{mock_app, MockRuntime};
use tauri::Manager;
use tempfile::tempdir;
use tokio::sync::Mutex;

// 创建测试环境，返回存储状态和临时目录
fn setup() -> (StorageState<MockRuntime>, tempfile::TempDir) {
    let temp_dir = tempdir().unwrap();
    let db = redb::Database::create(temp_dir.path().join("test.db")).unwrap();
    let app = mock_app();
    let app_handle = app.app_handle();
    let storage = Storage { db };
    let state = StorageState::<MockRuntime>(
        Mutex::new(storage),
        Mutex::new(App::<MockRuntime>::new(&app_handle)),
    );
    (state, temp_dir)
}

// 从存储状态获取存储引用
async fn get_storage(state: &StorageState<MockRuntime>) -> impl DerefMut<Target = Storage> + '_ {
    state.0.lock().await
}

#[tokio::test]
async fn test_basic_operations() {
    // 测试基本的 CRUD 操作
    let (state, _temp_dir) = setup();
    let mut storage = get_storage(&state).await;

    // 1. 创建和查询测试
    let list = List::new("Test List", "icon.png");
    let list_id = list.uuid.to_string();
    Repository::<List>::add(&mut storage, &list).unwrap();
    
    let result = Repository::<List>::get_by_name(&mut storage, &list_id).unwrap();
    assert!(result.is_some());
    assert_eq!(result.unwrap().title, "Test List");
    
    // 2. 更新测试
    Repository::<List>::update(&mut storage, &list_id, |l| {
        l.title = "Updated Title".to_string();
        Ok(())
    }).unwrap();
    
    let updated = Repository::<List>::get_by_name(&mut storage, &list_id).unwrap();
    assert!(updated.is_some());
    assert_eq!(updated.unwrap().title, "Updated Title");
    
    // 3. 过滤测试
    let list2 = List::new("Another List", "icon2.png");
    Repository::<List>::add(&mut storage, &list2).unwrap();
    
    let filtered = Repository::<List>::filter(&mut storage, |l| l.title.contains("Updated")).unwrap();
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].title, "Updated Title");
    
    // 4. 删除测试
    Repository::<List>::delete(&mut storage, &list_id).unwrap();
    assert!(Repository::<List>::get_by_name(&mut storage, &list_id).unwrap().is_none());
}

#[tokio::test]
async fn test_event_operations() {
    let (state, _temp_dir) = setup();
    let mut storage = get_storage(&state).await;

    // 创建和测试事件
    let event = Event::new("Test Event", "Test Content");
    let event_id = event.metadata.uuid.to_string();
    
    Repository::<Event>::add(&mut storage, &event).unwrap();
    let result = Repository::<Event>::get_by_name(&mut storage, &event_id).unwrap();
    
    assert!(result.is_some());
    assert_eq!(result.unwrap().title, "Test Event");
}

#[tokio::test]
async fn test_performance() {
    let (state, _temp_dir) = setup();
    let mut storage = get_storage(&state).await;
    
    // 批量插入测试 
    const ITEMS_COUNT: usize = 100;
    let start = Instant::now();
    
    for i in 1..=ITEMS_COUNT {
        let list = List::new(&format!("List {}", i), "icon.png");
        Repository::<List>::add(&mut storage, &list).unwrap();
    }
    
    // 批量读取测试
    let all_lists = Repository::<List>::get_all(&mut storage).unwrap();
    assert_eq!(all_lists.len(), ITEMS_COUNT);
    
    // 批量过滤测试
    let filtered = Repository::<List>::filter(&mut storage, |l| l.title.contains("5")).unwrap();
    assert!(filtered.len() > 0); // 应该至少找到 List 5, 15, 25...
    
    // 性能足够好，整个测试在合理时间内完成
    assert!(start.elapsed().as_secs() < 5); // 确保性能测试在5秒内完成
}
