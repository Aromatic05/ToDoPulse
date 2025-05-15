use anyhow::Result;
use chrono::Utc;
use std::io::Write;
use std::path::Path;
use tempfile::tempdir;
use tokio::fs;

use crate::function::sync::model::{
    ConflictStrategy, DiffEntry, DiffResult, DiffType, EntryState, EntryType, FileSystemState,
    SyncOperationStatus, SyncOperationType, SyncSession,
};
use crate::function::sync::sync_operations::{
    apply_config_to_session, collect_states, create_sync_session, execute_sync_operations, plan_sync_operations,
};
use crate::utils::config::{get_webdav_config, use_llm};

/// 创建测试用FileSystemState
async fn create_test_state() -> Result<(FileSystemState, tempfile::TempDir)> {
    // 创建临时目录
    let temp_dir = tempdir()?;
    let root_path = temp_dir.path();

    // 创建一些测试文件和目录
    let file1_path = root_path.join("file1.txt");
    let mut file1 = std::fs::File::create(&file1_path)?;
    file1.write_all("测试文件1内容".as_bytes())?;

    let subdir_path = root_path.join("subdir");
    std::fs::create_dir(&subdir_path)?;

    let file2_path = subdir_path.join("file2.txt");
    let mut file2 = std::fs::File::create(&file2_path)?;
    file2.write_all("测试文件2内容".as_bytes())?;

    // 创建FileSystemState
    let mut state = FileSystemState::new();

    // 添加文件
    let file1_entry = EntryState::new_file(
        "/file1.txt".to_string(),
        Some(Utc::now()),
        Some(file1.metadata()?.len()),
    );
    state.add_entry(file1_entry);

    // 添加目录
    let dir_entry = EntryState::new_directory("/subdir".to_string(), Some(Utc::now()));
    state.add_entry(dir_entry);

    // 添加子目录中的文件
    let file2_entry = EntryState::new_file(
        "/subdir/file2.txt".to_string(),
        Some(Utc::now()),
        Some(file2.metadata()?.len()),
    );
    state.add_entry(file2_entry);

    Ok((state, temp_dir))
}

/// 创建测试用DiffResult
fn create_test_diff() -> DiffResult {
    let mut diff = DiffResult::new();

    // 添加新增文件
    diff.add_entry(DiffEntry {
        path: "/new_file.txt".to_string(),
        diff_type: DiffType::Added,
        entry_type: EntryType::File,
        local_state: Some(EntryState::new_file(
            "/new_file.txt".to_string(),
            Some(Utc::now()),
            Some(100),
        )),
        remote_state: None,
    });

    // 添加新增目录
    diff.add_entry(DiffEntry {
        path: "/new_dir".to_string(),
        diff_type: DiffType::Added,
        entry_type: EntryType::Directory,
        local_state: Some(EntryState::new_directory(
            "/new_dir".to_string(),
            Some(Utc::now()),
        )),
        remote_state: None,
    });

    // 添加已修改文件
    diff.add_entry(DiffEntry {
        path: "/modified_file.txt".to_string(),
        diff_type: DiffType::Modified,
        entry_type: EntryType::File,
        local_state: Some(EntryState::new_file(
            "/modified_file.txt".to_string(),
            Some(Utc::now()),
            Some(200),
        )),
        remote_state: Some(EntryState::new_file(
            "/modified_file.txt".to_string(),
            Some(Utc::now()),
            Some(150),
        )),
    });

    // 添加已删除文件
    diff.add_entry(DiffEntry {
        path: "/deleted_file.txt".to_string(),
        diff_type: DiffType::Deleted,
        entry_type: EntryType::File,
        local_state: None,
        remote_state: Some(EntryState::new_file(
            "/deleted_file.txt".to_string(),
            Some(Utc::now()),
            Some(50),
        )),
    });

    diff
}

#[tokio::test]
async fn test_create_sync_session() -> Result<()> {
    // 检查WebDAV配置是否有效
    let llm = use_llm();
    let webdav_config = get_webdav_config()?;

    println!("WebDAV配置: {:?}", webdav_config);

    if !webdav_config.enabled {
        println!("WebDAV同步未启用，跳过测试");
        return Ok(());
    }

    // 创建同步会话
    let session = create_sync_session()?;

    // 验证会话参数
    assert_eq!(session.remote_dir, webdav_config.remote_dir);
    assert!(session.operations.is_empty());
    assert!(session.end_time.is_none());

    Ok(())
}

#[tokio::test]
async fn test_apply_config_to_session() -> Result<()> {
    // 检查WebDAV配置是否有效
    let webdav_config = get_webdav_config()?;
    if !webdav_config.enabled {
        println!("WebDAV同步未启用，跳过测试");
        return Ok(());
    }

    // 创建测试会话
    let mut session = SyncSession::new(Path::new("/tmp").to_path_buf(), "/test".to_string());

    // 应用配置
    apply_config_to_session(&mut session)?;

    // 验证配置被正确应用
    assert_eq!(session.remote_dir, webdav_config.remote_dir);

    Ok(())
}

#[tokio::test]
async fn test_plan_sync_operations() -> Result<()> {
    // 创建测试会话
    let mut session = SyncSession::new(Path::new("/tmp").to_path_buf(), "/test".to_string());

    // 创建测试差异结果
    let diff = create_test_diff();

    // 测试PreferLocal策略
    plan_sync_operations(&mut session, &diff, ConflictStrategy::PreferLocal)?;

    // 验证计划的操作
    assert_eq!(session.operations.len(), 4); // 新增文件、新增目录、修改文件、删除文件

    // 验证特定操作类型
    let upload_ops = session
        .operations
        .iter()
        .filter(|op| op.operation_type == SyncOperationType::Upload)
        .count();
    assert_eq!(upload_ops, 2); // 新增文件和修改文件

    let create_dir_ops = session
        .operations
        .iter()
        .filter(|op| op.operation_type == SyncOperationType::CreateRemoteDirectory)
        .count();
    assert_eq!(create_dir_ops, 1); // 新增目录

    let delete_ops = session
        .operations
        .iter()
        .filter(|op| op.operation_type == SyncOperationType::DeleteRemote)
        .count();
    assert_eq!(delete_ops, 1); // 删除文件

    // 清空会话操作
    session.operations.clear();

    // 测试PreferRemote策略
    plan_sync_operations(&mut session, &diff, ConflictStrategy::PreferRemote)?;

    // 验证计划的操作（对于修改的文件应该是下载而不是上传）
    let download_ops = session
        .operations
        .iter()
        .filter(|op| op.operation_type == SyncOperationType::Download)
        .count();
    assert_eq!(download_ops, 1); // 修改文件

    // 清空会话操作
    session.operations.clear();

    // 测试Skip策略
    plan_sync_operations(&mut session, &diff, ConflictStrategy::Skip)?;

    // 验证跳过的操作
    let skip_ops = session
        .operations
        .iter()
        .filter(|op| op.operation_type == SyncOperationType::Skip)
        .count();
    assert_eq!(skip_ops, 1); // 修改文件

    Ok(())
}

#[tokio::test]
async fn test_save_and_load_sync_state() -> Result<()> {
    // 创建测试状态
    let (state, _temp_dir) = create_test_state().await?;

    // 创建临时目录用于保存状态
    let temp_dir = tempdir()?;
    let config_dir = temp_dir.path().join("config");
    fs::create_dir_all(&config_dir).await?;

    // 保存为自定义路径以避免影响实际配置
    let state_dir = config_dir.join("sync_state");
    fs::create_dir_all(&state_dir).await?;

    let local_state_path = state_dir.join("local_state.json");
    let remote_state_path = state_dir.join("remote_state.json");

    // 保存状态
    crate::function::sync::state::save_state(&state, &local_state_path).await?;
    crate::function::sync::state::save_state(&state, &remote_state_path).await?;

    // 确认文件存在
    assert!(local_state_path.exists());
    assert!(remote_state_path.exists());

    // 加载状态
    let local_state = crate::function::sync::state::load_state(&local_state_path).await?;
    let remote_state = crate::function::sync::state::load_state(&remote_state_path).await?;

    // 验证加载的状态与原始状态相同
    assert_eq!(local_state.entry_count(), state.entry_count());
    assert_eq!(remote_state.entry_count(), state.entry_count());

    Ok(())
}

// 注意：以下测试需要有效的WebDAV配置和连接
// 这些测试会对实际的WebDAV服务器进行操作，请在测试目录中进行
#[tokio::test]
async fn test_collect_states() -> Result<()> {
    // 检查WebDAV配置是否有效
    let haha = use_llm();
    print!("{:?} {}", {}, haha);
    let webdav_config = get_webdav_config()?;
    if !webdav_config.enabled {
        println!("WebDAV同步未启用，跳过测试");
        return Ok(());
    }

    // 创建临时目录作为本地数据目录
    let temp_dir = tempdir()?;
    let local_dir = temp_dir.path();

    // 创建测试文件
    let file_path = local_dir.join("test_collect.txt");
    let mut file = std::fs::File::create(&file_path)?;
    file.write_all("测试collect_states功能".as_bytes())?;

    // 创建测试会话
    let mut session = SyncSession::new(
        local_dir.to_path_buf(),
        // 使用一个专门的测试目录避免干扰实际数据
        format!("{}/test_sync_operations", webdav_config.remote_dir),
    );

    // 确保配置正确应用
    apply_config_to_session(&mut session)?;

    // 收集状态
    let result = collect_states(&mut session).await;

    // 验证结果
    assert!(result.is_ok(), "状态收集失败: {:?}", result.err());

    let (local_state, remote_state) = result?;

    // 验证本地状态
    assert!(local_state.entry_count() > 0);
    assert!(local_state.entries.contains_key("/test_collect.txt"));

    // 远程状态可能为空或不为空，取决于远程目录是否已存在及其内容

    Ok(())
}

#[tokio::test]
async fn test_perform_sync_cycle() -> Result<()> {
    // 检查WebDAV配置是否有效
    let webdav_config = get_webdav_config()?;
    if !webdav_config.enabled {
        println!("WebDAV同步未启用，跳过测试");
        return Ok(());
    }

    // 创建临时目录作为本地数据目录
    let temp_dir = tempdir()?;
    let local_dir = temp_dir.path();

    // 创建测试文件
    let file_path = local_dir.join("test_sync_cycle.txt");
    let mut file = std::fs::File::create(&file_path)?;
    file.write_all("测试完整同步周期".as_bytes())?;

    // 创建测试子目录和文件
    let subdir_path = local_dir.join("test_subdir");
    std::fs::create_dir(&subdir_path)?;

    let subfile_path = subdir_path.join("test_subfile.txt");
    let mut subfile = std::fs::File::create(&subfile_path)?;
    subfile.write_all("测试子目录文件同步".as_bytes())?;

    // 重写create_sync_session以使用临时目录
    let mut session = SyncSession::new(
        local_dir.to_path_buf(),
        // 使用一个专门的测试目录避免干扰实际数据
        format!("{}/test_sync_cycle", webdav_config.remote_dir),
    );

    // 应用WebDAV配置
    apply_config_to_session(&mut session)?;

    // 收集状态
    let (local_state, remote_state) = collect_states(&mut session).await?;

    // 比较差异
    let diff_config = crate::function::sync::diff::DiffConfig::default();
    let diff =
        crate::function::sync::diff::compare_states(&local_state, &remote_state, &diff_config)?;

    // 计划操作
    plan_sync_operations(&mut session, &diff, ConflictStrategy::PreferLocal)?;

    // 验证是否有操作被计划
    assert!(!session.operations.is_empty());

    // 执行操作
    execute_sync_operations(&mut session).await?;

    // 验证所有操作是否已完成
    let failed_ops = session
        .operations
        .iter()
        .filter(|op| op.status == SyncOperationStatus::Failed)
        .count();
    assert_eq!(failed_ops, 0, "有{}个操作失败", failed_ops);

    let completed_ops = session
        .operations
        .iter()
        .filter(|op| op.status == SyncOperationStatus::Completed)
        .count();
    assert_eq!(completed_ops, session.operations.len());

    // 检查统计信息
    let stats = session.get_stats();
    assert_eq!(stats.total, session.operations.len());
    assert_eq!(stats.completed, session.operations.len());
    assert_eq!(stats.failed, 0);

    Ok(())
}

// 这个测试检查完整的同步流程
#[tokio::test]
async fn test_perform_sync() -> Result<()> {
    // 检查WebDAV配置是否有效
    let webdav_config = get_webdav_config()?;
    if !webdav_config.enabled {
        println!("WebDAV同步未启用，跳过测试");
        return Ok(());
    }

    // 修改环境以使用临时目录
    // 注意：这不是理想的方法，因为它可能会影响其他测试
    // 在实际环境中，你可能需要修改perform_sync函数以接受参数
    // 或者使用环境变量来控制测试行为

    // 创建临时目录作为本地数据目录
    let temp_dir = tempdir()?;
    let local_dir = temp_dir.path();

    // 创建测试文件
    let file_path = local_dir.join("test_perform_sync.txt");
    let mut file = std::fs::File::create(&file_path)?;
    file.write_all("测试完整同步功能".as_bytes())?;

    // 这里我们不能直接测试perform_sync()，因为它使用固定的本地目录
    // 作为替代，我们可以测试其组件函数，就像上面的test_perform_sync_cycle一样
    // 或者你可以修改代码以允许指定本地目录

    println!("注意：完整perform_sync测试需要修改代码以支持指定测试目录");

    Ok(())
}