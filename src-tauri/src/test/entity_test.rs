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
    
    // 使用 assert! 检查操作是否成功，而不是 .unwrap()
    assert!(Repository::<List>::add(&mut *storage, &list).is_ok());
    
    let result = Repository::<List>::get_by_name(&mut *storage, &list_id)
        .expect("获取列表失败"); // 使用 expect 提供更清晰的错误信息
    assert!(result.is_some(), "刚刚添加的列表应该能被找到");
    assert_eq!(result.unwrap().title, "Test List");
    
    // 2. 更新测试
    let update_result = Repository::<List>::update(&mut *storage, &list_id, |l| {
        l.title = "Updated Title".to_string();
        Ok(())
    });
    assert!(update_result.is_ok());
    
    let updated = Repository::<List>::get_by_name(&mut *storage, &list_id)
        .expect("获取更新后的列表失败");
    assert!(updated.is_some());
    assert_eq!(updated.unwrap().title, "Updated Title");
    
    // 3. 过滤测试
    let list2 = List::new("Another List", "icon2.png");
    assert!(Repository::<List>::add(&mut *storage, &list2).is_ok());
    
    let filtered = Repository::<List>::filter(&mut *storage, |l| l.title.contains("Updated"))
        .expect("过滤列表失败");
    assert_eq!(filtered.len(), 1, "应该只过滤出一个匹配的列表");
    assert_eq!(filtered[0].title, "Updated Title");
    
    // 4. 删除测试
    assert!(Repository::<List>::delete(&mut *storage, &list_id).is_ok());
    let deleted = Repository::<List>::get_by_name(&mut *storage, &list_id)
        .expect("获取已删除的列表失败");
    assert!(deleted.is_none(), "列表被删除后应该找不到");
}

#[tokio::test]
async fn test_event_operations() {
    let (state, _temp_dir) = setup();
    let mut storage = get_storage(&state).await;

    // 创建和测试事件
    let event = Event::new("Test Event", "Test Content");
    let event_id = event.metadata.uuid.to_string();
    
    assert!(Repository::<Event>::add(&mut *storage, &event).is_ok());
    
    let result = Repository::<Event>::get_by_name(&mut *storage, &event_id)
        .expect("获取事件失败");
    
    assert!(result.is_some(), "刚刚添加的事件应该能被找到");
    assert_eq!(result.unwrap().title, "Test Event");
}

#[tokio::test]
async fn test_performance() {
    let (state, _temp_dir) = setup();
    let mut storage = get_storage(&state).await;
    
    const ITEMS_COUNT: usize = 100;
    let start = Instant::now();
    
    // 批量插入测试
    // 提示：为了获得最佳性能，批量插入应该在单个写事务中完成。
    // 这里我们假设 Repository::add 每次都创建一个新事务。
    for i in 1..=ITEMS_COUNT {
        let list = List::new(&format!("List {}", i), "icon.png");
        assert!(Repository::<List>::add(&mut *storage, &list).is_ok());
    }
    
    // 批量读取测试
    let all_lists = Repository::<List>::get_all(&mut *storage).expect("获取所有列表失败");
    assert_eq!(all_lists.len(), ITEMS_COUNT);
    
    // 批量过滤测试
    let filtered = Repository::<List>::filter(&mut *storage, |l| l.title.contains("5"))
        .expect("过滤列表失败");
    // "List 5", "List 15", ..., "List 95" (10个)
    // "List 50" - "List 59" (10个)
    // "List 5" 和 "List 55" 被重复计算，所以总数是 10 + 10 - 1 = 19
    assert_eq!(filtered.len(), 19, "过滤结果的数量不正确");
    
    // 性能足够好，整个测试在合理时间内完成
    let duration = start.elapsed();
    println!("性能测试耗时: {:?}", duration);
    assert!(duration.as_secs() < 5, "性能测试应该在5秒内完成");
}
