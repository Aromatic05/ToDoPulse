use anyhow::{anyhow, Result};
use chrono::Utc;
use log::{debug, error, info, warn};
use std::path::Path;

use crate::function::sync::diff::{compare_states, DiffConfig};
use crate::function::sync::model::{
    ConflictStrategy, DiffResult, EntryState, FileSystemState, SyncOperation, SyncOperationStatus,
    SyncOperationType, SyncSession, SyncSessionStatus,
};
use crate::function::sync::state::{
    collect_local_state, load_state, save_state, StateCollectionConfig,
};
use crate::function::sync::webdav::{
    create_client, download_file, ensure_remote_dir_exists, upload_file,
};
use crate::utils::config::WebDav;
use crate::utils::path::AppPaths;

/// 将配置应用到会话
pub fn apply_config_to_session(session: &mut SyncSession) -> Result<()> {
    let webdav_config = WebDav::load()?;

    // 检查WebDAV是否启用
    if !webdav_config.enabled {
        return Err(anyhow!("WebDAV同步未启用"));
    }

    // 更新会话的远程目录
    session.remote_dir = webdav_config.remote_dir.clone();

    Ok(())
}

/// 创建一个新的同步会话
pub fn create_sync_session() -> Result<SyncSession> {
    // 获取本地数据目录
    let local_dir = AppPaths::data_dir().clone();

    // 从配置中获取远程目录
    let remote_dir = {
        let webdav_config = WebDav::load()?;
        if webdav_config.enabled {
            webdav_config.remote_dir.clone()
        } else {
            "/ToDoPulse".to_string()
        }
    };

    // 创建会话
    let mut session = SyncSession::new(local_dir, remote_dir);

    // 更新配置
    match apply_config_to_session(&mut session) {
        Ok(_) => {}
        Err(e) => {
            session.fail(format!("配置应用失败: {}", e));
            return Ok(session);
        }
    }

    Ok(session)
}

/// 收集本地和远程状态
pub async fn collect_states(
    session: &mut SyncSession,
) -> Result<(FileSystemState, FileSystemState)> {
    session.status = SyncSessionStatus::CollectingState;

    // 获取WebDAV客户端配置
    let webdav_config = WebDav::load()?;

    // 检查WebDAV是否启用
    if !webdav_config.enabled {
        return Err(anyhow!("WebDAV同步未启用"));
    }

    // 创建WebDAV客户端
    let client = create_client(
        &webdav_config.host,
        &webdav_config.username,
        &webdav_config.password,
    )
    .await?;

    // 收集本地状态
    info!("收集本地文件系统状态: {}", session.local_dir.display());
    let state_config = StateCollectionConfig::default();
    let local_state = collect_local_state(&session.local_dir, &state_config).await?;

    // 收集远程状态
    info!("收集远程文件系统状态: {}", session.remote_dir);
    let webdav_remote_state =
        crate::function::sync::webdav::collect_remote_state(&client, &session.remote_dir).await?;

    // 将webdav::FileSystemState转换为model::FileSystemState
    let mut remote_state = FileSystemState::new();
    remote_state.collection_time = webdav_remote_state.collection_time;

    for (path, webdav_entry) in &webdav_remote_state.entries {
        if webdav_entry.is_file() {
            let entry =
                EntryState::new_file(path.clone(), webdav_entry.modified, webdav_entry.size);
            remote_state.add_entry(entry);
        } else {
            let entry = EntryState::new_directory(path.clone(), webdav_entry.modified);
            remote_state.add_entry(entry);
        }
    }

    Ok((local_state, remote_state))
}

/// 根据差异结果计划同步操作
pub fn plan_sync_operations(
    session: &mut SyncSession,
    diff: &DiffResult,
    conflict_strategy: ConflictStrategy,
) -> Result<()> {
    session.status = SyncSessionStatus::Planning;
    info!("计划同步操作");

    // 首先处理目录操作，确保目录结构存在
    for entry in &diff.entries {
        if entry.is_directory() {
            match entry.diff_type {
                crate::function::sync::model::DiffType::Added => {
                    // 本地添加的目录，需要在远程创建
                    if entry.local_state.is_some() {
                        session.add_operation(SyncOperation {
                            operation_type: SyncOperationType::CreateRemoteDirectory,
                            path: entry.path.clone(),
                            entry_type: entry.entry_type.clone(),
                            status: SyncOperationStatus::Pending,
                            error: None,
                        });
                    }
                }
                crate::function::sync::model::DiffType::Deleted => {
                    // 本地删除的目录，需要在远程删除
                    if entry.remote_state.is_some() {
                        session.add_operation(SyncOperation {
                            operation_type: SyncOperationType::DeleteRemote,
                            path: entry.path.clone(),
                            entry_type: entry.entry_type.clone(),
                            status: SyncOperationStatus::Pending,
                            error: None,
                        });
                    }
                }
                _ => {} // 对于目录，其他差异类型不需要特殊处理
            }
        }
    }

    // 然后处理文件操作
    for entry in &diff.entries {
        if entry.is_file() {
            match entry.diff_type {
                crate::function::sync::model::DiffType::Added => {
                    // 本地添加的文件，需要上传
                    if entry.local_state.is_some() {
                        session.add_operation(SyncOperation {
                            operation_type: SyncOperationType::Upload,
                            path: entry.path.clone(),
                            entry_type: entry.entry_type.clone(),
                            status: SyncOperationStatus::Pending,
                            error: None,
                        });
                    }
                }
                crate::function::sync::model::DiffType::Deleted => {
                    // 本地删除的文件，远程存在，需要删除远程文件
                    if entry.remote_state.is_some() {
                        session.add_operation(SyncOperation {
                            operation_type: SyncOperationType::DeleteRemote,
                            path: entry.path.clone(),
                            entry_type: entry.entry_type.clone(),
                            status: SyncOperationStatus::Pending,
                            error: None,
                        });
                    }
                }
                crate::function::sync::model::DiffType::Modified => {
                    // 文件被修改，根据冲突策略决定操作
                    match conflict_strategy {
                        ConflictStrategy::PreferLocal => {
                            // 优先使用本地版本，上传
                            session.add_operation(SyncOperation {
                                operation_type: SyncOperationType::Upload,
                                path: entry.path.clone(),
                                entry_type: entry.entry_type.clone(),
                                status: SyncOperationStatus::Pending,
                                error: None,
                            });
                        }
                        ConflictStrategy::PreferRemote => {
                            // 优先使用远程版本，下载
                            session.add_operation(SyncOperation {
                                operation_type: SyncOperationType::Download,
                                path: entry.path.clone(),
                                entry_type: entry.entry_type.clone(),
                                status: SyncOperationStatus::Pending,
                                error: None,
                            });
                        }
                        ConflictStrategy::KeepBoth => {
                            // 保留两者，下载远程版本并重命名
                            let ext = Path::new(&entry.path)
                                .extension()
                                .and_then(|e| e.to_str())
                                .unwrap_or("");

                            let stem = Path::new(&entry.path)
                                .file_stem()
                                .and_then(|s| s.to_str())
                                .unwrap_or("");

                            let parent = Path::new(&entry.path)
                                .parent()
                                .and_then(|p| p.to_str())
                                .unwrap_or("");

                            let timestamp = Utc::now().timestamp();
                            let new_path = if ext.is_empty() {
                                format!("{}/{}_remote_{}", parent, stem, timestamp)
                            } else {
                                format!("{}/{}_remote_{}.{}", parent, stem, timestamp, ext)
                            };

                            // 用新路径下载远程文件
                            session.add_operation(SyncOperation {
                                operation_type: SyncOperationType::Download,
                                path: new_path,
                                entry_type: entry.entry_type.clone(),
                                status: SyncOperationStatus::Pending,
                                error: None,
                            });
                        }
                        ConflictStrategy::Skip => {
                            // 跳过冲突文件
                            session.add_operation(SyncOperation {
                                operation_type: SyncOperationType::Skip,
                                path: entry.path.clone(),
                                entry_type: entry.entry_type.clone(),
                                status: SyncOperationStatus::Skipped,
                                error: Some("冲突文件已跳过".to_string()),
                            });
                        }
                    }
                }
                _ => {} // 其他情况不需要处理
            }
        }
    }

    info!("计划了 {} 个同步操作", session.operations.len());
    Ok(())
}

/// 执行同步操作
pub async fn execute_sync_operations(session: &mut SyncSession) -> Result<()> {
    session.status = SyncSessionStatus::Executing;
    info!("执行同步操作");

    // 获取WebDAV客户端配置
    let webdav_config = WebDav::load()?;

    // 检查WebDAV是否启用
    if !webdav_config.enabled {
        return Err(anyhow!("WebDAV同步未启用"));
    }

    // 创建WebDAV客户端
    let client = create_client(
        &webdav_config.host,
        &webdav_config.username,
        &webdav_config.password,
    )
    .await?;

    // 确保远程根目录存在
    ensure_remote_dir_exists(&client, &session.remote_dir).await?;

    // 执行每个操作
    for operation in &mut session.operations {
        if operation.status != SyncOperationStatus::Pending {
            continue;
        }

        // 更新状态为执行中
        operation.status = SyncOperationStatus::InProgress;

        match operation.operation_type {
            SyncOperationType::Upload => {
                info!("上传文件: {}", operation.path);
                let local_path = session
                    .local_dir
                    .join(operation.path.trim_start_matches('/'));
                let remote_path = format!("{}{}", session.remote_dir, operation.path);

                match upload_file(&client, &local_path, &remote_path).await {
                    Ok(_) => {
                        operation.status = SyncOperationStatus::Completed;
                    }
                    Err(e) => {
                        operation.status = SyncOperationStatus::Failed;
                        operation.error = Some(format!("上传失败: {}", e));
                        error!("上传文件失败 {}: {}", operation.path, e);
                    }
                }
            }
            SyncOperationType::Download => {
                info!("下载文件: {}", operation.path);
                let local_path = session
                    .local_dir
                    .join(operation.path.trim_start_matches('/'));
                let remote_path = format!("{}{}", session.remote_dir, operation.path);

                match download_file(&client, &remote_path, &local_path).await {
                    Ok(_) => {
                        operation.status = SyncOperationStatus::Completed;
                    }
                    Err(e) => {
                        operation.status = SyncOperationStatus::Failed;
                        operation.error = Some(format!("下载失败: {}", e));
                        error!("下载文件失败 {}: {}", operation.path, e);
                    }
                }
            }
            SyncOperationType::DeleteLocal => {
                info!("删除本地文件: {}", operation.path);
                let local_path = session
                    .local_dir
                    .join(operation.path.trim_start_matches('/'));

                match tokio::fs::remove_file(&local_path).await {
                    Ok(_) => {
                        operation.status = SyncOperationStatus::Completed;
                    }
                    Err(e) => {
                        operation.status = SyncOperationStatus::Failed;
                        operation.error = Some(format!("删除失败: {}", e));
                        error!("删除本地文件失败 {}: {}", operation.path, e);
                    }
                }
            }
            SyncOperationType::DeleteRemote => {
                info!("删除远程文件: {}", operation.path);
                let remote_path = format!("{}{}", session.remote_dir, operation.path);

                match client.delete(&remote_path).await {
                    Ok(_) => {
                        operation.status = SyncOperationStatus::Completed;
                    }
                    Err(e) => {
                        operation.status = SyncOperationStatus::Failed;
                        operation.error = Some(format!("删除失败: {}", e));
                        error!("删除远程文件失败 {}: {}", operation.path, e);
                    }
                }
            }
            SyncOperationType::CreateLocalDirectory => {
                info!("创建本地目录: {}", operation.path);
                let local_path = session
                    .local_dir
                    .join(operation.path.trim_start_matches('/'));

                match tokio::fs::create_dir_all(&local_path).await {
                    Ok(_) => {
                        operation.status = SyncOperationStatus::Completed;
                    }
                    Err(e) => {
                        operation.status = SyncOperationStatus::Failed;
                        operation.error = Some(format!("创建失败: {}", e));
                        error!("创建本地目录失败 {}: {}", operation.path, e);
                    }
                }
            }
            SyncOperationType::CreateRemoteDirectory => {
                info!("创建远程目录: {}", operation.path);
                let remote_path = format!("{}{}", session.remote_dir, operation.path);

                match ensure_remote_dir_exists(&client, &remote_path).await {
                    Ok(_) => {
                        operation.status = SyncOperationStatus::Completed;
                    }
                    Err(e) => {
                        operation.status = SyncOperationStatus::Failed;
                        operation.error = Some(format!("创建失败: {}", e));
                        error!("创建远程目录失败 {}: {}", operation.path, e);
                    }
                }
            }
            SyncOperationType::Skip => {
                // 已经在计划阶段标记为跳过
                debug!("跳过操作: {}", operation.path);
            }
        }
    }

    // 更新会话状态
    let failed_count = session
        .operations
        .iter()
        .filter(|op| op.status == SyncOperationStatus::Failed)
        .count();

    if failed_count > 0 {
        session.status = SyncSessionStatus::Failed;
        session.error = Some(format!("同步过程中有 {} 个操作失败", failed_count));
    } else {
        session.status = SyncSessionStatus::Completed;
    }

    session.end_time = Some(Utc::now());

    let stats = session.get_stats();
    info!(
        "同步完成: 总操作 {}, 成功 {}, 失败 {}, 跳过 {}, 用时 {}秒",
        stats.total, stats.completed, stats.failed, stats.skipped, stats.duration_seconds
    );

    Ok(())
}

/// 执行完整的同步流程
pub async fn perform_sync() -> Result<SyncSession> {
    // 创建同步会话
    let mut session = create_sync_session()?;

    // 收集状态
    let (local_state, remote_state) = match collect_states(&mut session).await {
        Ok(states) => states,
        Err(e) => {
            session.fail(format!("状态收集失败: {}", e));
            return Ok(session);
        }
    };

    // 比较差异
    session.status = SyncSessionStatus::Diffing;
    let diff_config = DiffConfig::default();
    let diff = match compare_states(&local_state, &remote_state, &diff_config) {
        Ok(diff) => diff,
        Err(e) => {
            session.fail(format!("差异比较失败: {}", e));
            return Ok(session);
        }
    };

    // 计划操作
    match plan_sync_operations(&mut session, &diff, ConflictStrategy::PreferLocal) {
        Ok(_) => {}
        Err(e) => {
            session.fail(format!("操作计划失败: {}", e));
            return Ok(session);
        }
    }

    // 如果没有操作需要执行，直接完成
    if session.operations.is_empty() {
        session.status = SyncSessionStatus::Completed;
        session.end_time = Some(Utc::now());
        info!("没有需要同步的内容");
        return Ok(session);
    }

    // 执行操作
    match execute_sync_operations(&mut session).await {
        Ok(_) => {}
        Err(e) => {
            session.fail(format!("操作执行失败: {}", e));
            return Ok(session);
        }
    }

    Ok(session)
}

/// 保存和加载同步状态
#[allow(dead_code)]
pub async fn save_sync_state(
    local_state: &FileSystemState,
    remote_state: &FileSystemState,
) -> Result<()> {
    let state_dir = AppPaths::config_dir().join("sync_state");
    tokio::fs::create_dir_all(&state_dir).await?;

    let local_state_path = state_dir.join("local_state.json");
    let remote_state_path = state_dir.join("remote_state.json");

    save_state(local_state, &local_state_path).await?;
    save_state(remote_state, &remote_state_path).await?;

    Ok(())
}

/// 加载同步状态
#[allow(dead_code)]
pub async fn load_sync_state() -> Result<(Option<FileSystemState>, Option<FileSystemState>)> {
    let state_dir = AppPaths::config_dir().join("sync_state");
    let local_state_path = state_dir.join("local_state.json");
    let remote_state_path = state_dir.join("remote_state.json");

    let local_state = if local_state_path.exists() {
        match load_state(&local_state_path).await {
            Ok(state) => Some(state),
            Err(e) => {
                warn!("无法加载本地状态: {}", e);
                None
            }
        }
    } else {
        None
    };

    let remote_state = if remote_state_path.exists() {
        match load_state(&remote_state_path).await {
            Ok(state) => Some(state),
            Err(e) => {
                warn!("无法加载远程状态: {}", e);
                None
            }
        }
    } else {
        None
    };

    Ok((local_state, remote_state))
}
