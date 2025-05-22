use crate::function::sync::model::{EntryState, FileSystemState};
use crate::function::sync::state::{
    collect_local_state, compute_file_hash, load_state, save_state, should_exclude,
    system_time_to_datetime, StateCollectionConfig,
};

use anyhow::Result;
use chrono::Utc;
use std::io::Write;
use std::path::Path;
use tempfile::tempdir;

#[test]
fn test_should_exclude() {
    // 测试准确匹配
    let path = Path::new("some/path/.git");
    assert!(should_exclude(path, &vec![".git".to_string()]));

    // 测试前缀通配符
    let path = Path::new("some/path/document.tmp");
    assert!(should_exclude(path, &vec!["*.tmp".to_string()]));

    // 测试后缀通配符
    let path = Path::new("some/path/~document");
    assert!(should_exclude(path, &vec!["~*".to_string()]));

    // 测试不匹配的情况
    let path = Path::new("some/path/document.txt");
    assert!(!should_exclude(
        path,
        &vec![".git".to_string(), "*.tmp".to_string()]
    ));
}

#[tokio::test]
async fn test_compute_file_hash() -> Result<()> {
    // 创建临时目录
    let temp_dir = tempdir()?;
    let file_path = temp_dir.path().join("test_file.txt");

    // 写入一些内容
    let content = "Hello, world!";
    let mut file = std::fs::File::create(&file_path)?;
    file.write_all(content.as_bytes())?;

    // 计算哈希并验证结果不为空
    let hash = compute_file_hash(&file_path).await?;
    assert!(!hash.is_empty());

    // 计算相同内容的哈希应该得到相同结果
    let another_file_path = temp_dir.path().join("another_test_file.txt");
    let mut another_file = std::fs::File::create(&another_file_path)?;
    another_file.write_all(content.as_bytes())?;

    let another_hash = compute_file_hash(&another_file_path).await?;
    assert_eq!(hash, another_hash);

    // 修改内容，哈希应该不同
    let mut modified_file = std::fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(&file_path)?;
    modified_file.write_all("Modified content".as_bytes())?;

    let modified_hash = compute_file_hash(&file_path).await?;
    assert_ne!(hash, modified_hash);

    Ok(())
}

#[tokio::test]
async fn test_collect_local_state() -> Result<()> {
    // 创建临时目录
    let temp_dir = tempdir()?;
    let root_path = temp_dir.path();

    // 创建一些文件和目录
    let file1_path = root_path.join("file1.txt");
    let mut file1 = std::fs::File::create(&file1_path)?;
    file1.write_all(b"File 1 content")?;

    let subdir_path = root_path.join("subdir");
    std::fs::create_dir(&subdir_path)?;

    let file2_path = subdir_path.join("file2.txt");
    let mut file2 = std::fs::File::create(&file2_path)?;
    file2.write_all(b"File 2 content")?;

    // 创建一个应该被排除的文件
    let excluded_path = root_path.join(".DS_Store");
    let mut excluded_file = std::fs::File::create(&excluded_path)?;
    excluded_file.write_all(b"Some content")?;

    // 收集状态
    let config = StateCollectionConfig::default();
    let state = collect_local_state(root_path, &config).await?;

    // 验证结果
    assert_eq!(state.entry_count(), 3); // 根目录不计入，所以是3个条目：file1.txt, subdir, subdir/file2.txt

    // 检查文件项是否正确
    assert!(
        state
            .entries
            .get("/file1.txt")
            .map_or(false, |e| e.is_file()),
        "file1.txt 应该是文件"
    );
    assert!(
        state
            .entries
            .get("/subdir/file2.txt")
            .map_or(false, |e| e.is_file()),
        "subdir/file2.txt 应该是文件"
    );
    assert!(
        state
            .entries
            .get("/subdir")
            .map_or(false, |e| e.is_directory()),
        "subdir 应该是目录"
    );

    // 现在断言文件数量
    assert_eq!(
        state.file_count(),
        2,
        "应该有两个文件：/file1.txt 和 /subdir/file2.txt"
    );
    assert_eq!(state.directory_count(), 1); // subdir

    // 验证路径正确性
    assert!(state.entries.contains_key("/file1.txt"));
    assert!(state.entries.contains_key("/subdir"));
    assert!(state.entries.contains_key("/subdir/file2.txt"));

    // 验证被排除的文件不在结果中
    assert!(!state.entries.contains_key("/.DS_Store"));

    // 测试有限深度的情况
    let limited_config = StateCollectionConfig {
        max_depth: Some(0),
        ..Default::default()
    };
    let limited_state = collect_local_state(root_path, &limited_config).await?;

    // 应该只包含根目录下的条目，而不包含子目录中的内容
    assert_eq!(limited_state.entry_count(), 2); // file1.txt, subdir
    assert_eq!(limited_state.file_count(), 1);
    assert_eq!(limited_state.directory_count(), 1);

    Ok(())
}

#[tokio::test]
async fn test_save_and_load_state() -> Result<()> {
    // 创建一些测试状态
    let mut state = FileSystemState::new();

    let file_entry =
        EntryState::new_file("/test/file.txt".to_string(), Some(Utc::now()), Some(1024))
            .with_hash("abcdef1234567890".to_string());

    let dir_entry = EntryState::new_directory("/test/dir".to_string(), Some(Utc::now()));

    state.add_entry(file_entry);
    state.add_entry(dir_entry);

    // 保存到临时文件
    let temp_dir = tempdir()?;
    let state_file_path = temp_dir.path().join("state.json");

    save_state(&state, &state_file_path).await?;

    // 确认文件存在
    assert!(state_file_path.exists());

    // 加载状态
    let loaded_state = load_state(&state_file_path).await?;

    // 验证加载的状态与原始状态相同
    assert_eq!(loaded_state.entry_count(), state.entry_count());
    assert_eq!(loaded_state.file_count(), state.file_count());
    assert_eq!(loaded_state.directory_count(), state.directory_count());

    // 验证具体条目
    assert!(loaded_state.entries.contains_key("/test/file.txt"));
    assert!(loaded_state.entries.contains_key("/test/dir"));

    let loaded_file_entry = &loaded_state.entries["/test/file.txt"];
    assert_eq!(loaded_file_entry.path, "/test/file.txt");
    assert_eq!(loaded_file_entry.size, Some(1024));
    assert_eq!(
        loaded_file_entry.content_hash,
        Some("abcdef1234567890".to_string())
    );
    assert!(loaded_file_entry.is_file());

    let loaded_dir_entry = &loaded_state.entries["/test/dir"];
    assert_eq!(loaded_dir_entry.path, "/test/dir");
    assert!(loaded_dir_entry.is_directory());

    Ok(())
}

#[tokio::test]
async fn test_system_time_to_datetime() {
    // 创建一个SystemTime
    let now = std::time::SystemTime::now();

    // 转换为DateTime
    let datetime = system_time_to_datetime(now).unwrap();

    // 确保它是一个有效的DateTime
    assert!(!datetime.timestamp().is_negative());

    // 由于精度限制，直接比较可能不准确，但应该非常接近
    let now_ts = now.duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
    let dt_ts = datetime.timestamp() as u64;

    // 允许1秒的差异（由于不同时间表示方式的转换可能有微小差异）
    assert!((now_ts as i64 - dt_ts as i64).abs() <= 1);
}

#[tokio::test]
async fn test_state_collection_config_default() {
    // 测试StateCollectionConfig的默认值
    let config = StateCollectionConfig::default();

    // 验证默认值
    assert_eq!(config.compute_hash, false);
    assert!(config.exclusion_patterns.contains(&".DS_Store".to_string()));
    assert!(config.exclusion_patterns.contains(&"Thumbs.db".to_string()));
    assert!(config.exclusion_patterns.contains(&".git".to_string()));
    assert_eq!(config.max_depth, None);
}

#[tokio::test]
async fn test_collect_state_with_hash() -> Result<()> {
    // 创建临时目录
    let temp_dir = tempdir()?;
    let root_path = temp_dir.path();

    // 创建测试文件
    let file_path = root_path.join("file_with_hash.txt");
    let mut file = std::fs::File::create(&file_path)?;
    file.write_all("测试内容哈希计算".as_bytes())?;

    // 使用启用了哈希计算的配置
    let config = StateCollectionConfig {
        compute_hash: true,
        ..Default::default()
    };

    // 收集状态
    let state = collect_local_state(root_path, &config).await?;

    // 验证收集到了状态
    assert_eq!(state.entry_count(), 1);

    // 验证文件哈希
    let file_entry = state
        .entries
        .get("/file_with_hash.txt")
        .expect("文件应该存在");
    assert!(file_entry.content_hash.is_some(), "应该计算哈希值");
    assert!(
        !file_entry.content_hash.as_ref().unwrap().is_empty(),
        "哈希值不应为空"
    );

    Ok(())
}

#[tokio::test]
async fn test_collect_state_edge_cases() -> Result<()> {
    // 测试不存在的目录
    let non_existent_path = Path::new("/non_existent_dir_for_test");
    let config = StateCollectionConfig::default();

    let result = collect_local_state(non_existent_path, &config).await;
    assert!(result.is_err(), "应该返回错误，因为目录不存在");

    // 测试空目录
    let temp_dir = tempdir()?;
    let empty_dir = temp_dir.path();

    let state = collect_local_state(empty_dir, &config).await?;
    assert_eq!(state.entry_count(), 0, "空目录应该没有条目");

    Ok(())
}
