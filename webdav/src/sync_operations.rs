use anyhow::{anyhow, Result};
use chrono::Utc;
use core::panic;
use log::{error, warn};
use reqwest_dav::Client;
use std::path::{Path, PathBuf};

use crate::diff::{compare_states, DiffConfig};
use crate::manager::{with_app_path, with_config};
use crate::model::*;
use crate::path_resolver::PathResolver;
use crate::state::{collect_local_state, load_state, save_state, StateCollectionConfig};
use crate::webdav::*;
use crate::{with_config_if_enabled, without_config_enabled};

pub trait PathExt {
    fn contains(&self, pattern: &str) -> bool;
}

impl PathExt for PathBuf {
    fn contains(&self, pattern: &str) -> bool {
        self.to_string_lossy().contains(pattern)
    }
}

impl PathExt for Path {
    fn contains(&self, pattern: &str) -> bool {
        self.to_string_lossy().contains(pattern)
    }
}

/// 将配置应用到会话
pub fn apply_config_to_session(session: &mut SyncSession) -> Result<()> {
    with_config_if_enabled!(
      |config| {
        Ok(session.remote_dir = config.remote_dir().to_path_buf())
      },
      else Err(anyhow!("WebDAV配置未加载"))
    )
}

/// 创建一个新的同步会话
pub fn create_sync_session() -> Result<SyncSession> {
    // 获取本地数据目录
    let local_dir = with_app_path(|app| app.data_dir().clone());

    // 从配置中获取远程目录
    let remote_dir = with_config_if_enabled!(
      |config| {
        config.remote_dir().to_path_buf()
      },
      else "/ToDoPulse".into()
    );

    // 创建会话
    let mut session = SyncSession::new(local_dir, remote_dir);

    // 更新配置
    apply_config_to_session(&mut session).map_err(|e| anyhow!("应用配置到会话失败: {}", e))?;

    Ok(session)
}

/// 收集本地和远程状态
pub async fn collect_states(
    session: &mut SyncSession,
) -> Result<(FileSystemState, FileSystemState)> {
    // get client
    let client = client().await?;

    session.status = SyncSessionStatus::CollectingState;

    // 收集本地状态
    let state_config = StateCollectionConfig::default();
    let local_state = collect_local_state(&session.local_dir, &state_config).await?;

    // 收集远程状态
    let webdav_remote_state = collect_remote_state(&client, &session.remote_dir).await?;
    let mut remote_state = FileSystemState::new();
    remote_state.collection_time = webdav_remote_state.collection_time;

    for (webdav_path, webdav_entry) in &webdav_remote_state.entries {
        path_resolve(webdav_path, webdav_entry, &mut remote_state);
    }

    // 最终验证
    if let Some((path, _)) = remote_state
        .entries
        .iter()
        .find(|(path, _)| path.to_string_lossy().contains("webdav"))
    {
        panic!("状态收集失败,路径包含webdav信息: '{}'", path.display());
    }

    Ok((local_state, remote_state))
}

// 辅助函数
async fn client() -> Result<Box<Client>> {
    let maybe_creds = with_config(|config| {
        if config.enabled() {
            let (host, user, pass) = config.credential();

            Some((host.to_owned(), user.to_owned(), pass.to_owned()))
        } else {
            None
        }
    });

    let Some(creds) = maybe_creds else {
        return Err(anyhow!("WebDAV配置未启用"));
    };

    let client = create_client((&creds.0, &creds.1, &creds.2)).await?;

    Ok(Box::new(client))
}

/// 执行同步操作
pub async fn execute_sync_operations(
    mut session: SyncSession,
    client: &Client,
) -> Result<SyncSession> {
    session.status = SyncSessionStatus::Executing;

    without_config_enabled!(return Err(anyhow!("WebDAV配置未启用，无法执行同步操作")));

    // 确保远程目录存在
    ensure_remote_dir_exists(&client, &session.remote_dir).await?;

    for operation in &mut session.operations {
        if operation.status != SyncOperationStatus::Pending {
            continue;
        }

        operation.status = SyncOperationStatus::InProgress;

        // operation.path 应该是相对路径，如 "ToDoPulse/dffff/dffff.md"
        let relative_path = operation
        .path
        .strip_prefix("/")
        .unwrap_or(&operation.path);

        // 从相对路径中移除 ToDoPulse 前缀来构建本地路径
        let local_relative_path = relative_path
            .strip_prefix("ToDoPulse/")
            .unwrap_or(relative_path);

        match operation.operation_type {
            SyncOperationType::Download => {
                let local_path = session.local_dir.join(local_relative_path);
                let remote_path = Path::new("/").join(relative_path);

                // 确保父目录存在
                if let Some(parent) = local_path.parent() {
                    if let Err(e) = tokio::fs::create_dir_all(parent).await {
                        operation.status = SyncOperationStatus::Failed;
                        operation.error = Some(format!("创建父目录失败: {}", e));
                        continue;
                    }
                }

                match download_file(&client, &remote_path, &local_path).await {
                    Ok(_) => {
                        operation.status = SyncOperationStatus::Completed;
                    }
                    Err(e) => {
                        operation.status = SyncOperationStatus::Failed;
                        operation.error = Some(format!("下载失败: {}", e));
                        error!("下载失败 {}: {}", relative_path.display(), e);
                    }
                }
            }
            SyncOperationType::Upload => {
                let local_path = session.local_dir.join(local_relative_path);
                let remote_path = Path::new("/").join(relative_path);

                if !local_path.exists() {
                    operation.status = SyncOperationStatus::Failed;
                    operation.error = Some(format!("本地文件不存在: {}", local_path.display()));
                    continue;
                }

                match upload_file(&client, &local_path, &remote_path).await {
                    Ok(_) => {
                        operation.status = SyncOperationStatus::Completed;
                    }
                    Err(e) => {
                        operation.status = SyncOperationStatus::Failed;
                        operation.error = Some(format!("上传失败: {}", e));
                        error!("上传失败 {}: {}", relative_path.display(), e);
                    }
                }
            }
            SyncOperationType::CreateLocalDirectory => {
                let local_path = session.local_dir.join(local_relative_path);

                match tokio::fs::create_dir_all(&local_path).await {
                    Ok(_) => {
                        operation.status = SyncOperationStatus::Completed;
                    }
                    Err(e) => {
                        operation.status = SyncOperationStatus::Failed;
                        operation.error = Some(format!("创建失败: {}", e));
                    }
                }
            }
            SyncOperationType::CreateRemoteDirectory => {
                let remote_path = Path::new("/").join(relative_path);

                match ensure_remote_dir_exists(&client, &remote_path).await {
                    Ok(_) => {
                        operation.status = SyncOperationStatus::Completed;
                    }
                    Err(e) => {
                        operation.status = SyncOperationStatus::Failed;
                        operation.error = Some(format!("创建失败: {}", e));
                    }
                }
            }
            _ => {
                operation.status = SyncOperationStatus::Skipped;
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

    Ok(session)
}

/// 执行完整的同步流程
pub async fn perform_sync() -> Result<SyncSession> {
    let client = client().await?;
    // 创建同步会话
    let mut session = create_sync_session()?;

    // 尝试加载上一次的状态
    let states = load_sync_state()
        .await
        .map_err(|e| {
            warn!("无法加载上一次的同步状态: {}", e);
            anyhow!("同步状态加载失败")
        })
        .unwrap_or((None, None));

    let (saved_local_state, saved_remote_state) = states;

    // 收集当前状态
    let (local_state, remote_state) = collect_states(&mut session).await?;

    // 保存当前状态供下次使用
    if let Err(e) = save_sync_state(&local_state, &remote_state).await {
        warn!("保存同步状态失败: {}", e);
    }

    // 比较差异
    session.status = SyncSessionStatus::Diffing;
    let diff_config = DiffConfig::default();

    let diff =
        if let (Some(saved_local), Some(saved_remote)) = (saved_local_state, saved_remote_state) {
            let local_changes = compare_states(&saved_local, &local_state, &diff_config)?;

            let remote_changes = compare_states(&saved_remote, &remote_state, &diff_config)?;

            let mut combined_diff = local_changes;
            combined_diff
                .entries
                .extend(remote_changes.entries.iter().cloned());
            combined_diff
        } else {
            compare_states(&local_state, &remote_state, &diff_config)?
        };

    plan_sync_operations(&mut session, &diff, ConflictStrategy::PreferLocal)?;

    // 如果没有操作需要执行，直接完成
    if session.operations.is_empty() {
        session.status = SyncSessionStatus::Completed;
        session.end_time = Some(Utc::now());
        return Ok(session);
    }

    execute_sync_operations(session, &client).await
}

/// 保存和加载同步状态
pub async fn save_sync_state(
    local_state: &FileSystemState,
    remote_state: &FileSystemState,
) -> Result<()> {
    let state_dir = with_app_path(|app| app.config_dir().join("sync_state"));
    if let Err(e) = tokio::fs::create_dir_all(&state_dir).await {
        warn!("创建同步状态目录失败: {}", e);
        return Err(anyhow!("无法创建同步状态目录: {}", e));
    }

    // 创建临时文件
    let local_state_path = state_dir.join("local_state.json");
    let remote_state_path = state_dir.join("remote_state.json");
    let local_state_temp = state_dir.join("local_state.json.tmp");
    let remote_state_temp = state_dir.join("remote_state.json.tmp");

    // 先写入临时文件
    if let Err(e) = save_state(local_state, &local_state_temp).await {
        warn!("保存本地状态到临时文件失败: {}", e);
        return Err(anyhow!("保存本地状态失败: {}", e));
    }

    if let Err(e) = save_state(remote_state, &remote_state_temp).await {
        warn!("保存远程状态到临时文件失败: {}", e);
        // 清理本地临时文件
        let _ = tokio::fs::remove_file(&local_state_temp).await;
        return Err(anyhow!("保存远程状态失败: {}", e));
    }

    // 原子地替换正式文件
    if let Err(e) = tokio::fs::rename(&local_state_temp, &local_state_path).await {
        warn!("更新本地状态文件失败: {}", e);
        // 清理临时文件
        let _ = tokio::fs::remove_file(&local_state_temp).await;
        let _ = tokio::fs::remove_file(&remote_state_temp).await;
        return Err(anyhow!("无法更新本地状态文件: {}", e));
    }

    if let Err(e) = tokio::fs::rename(&remote_state_temp, &remote_state_path).await {
        warn!("更新远程状态文件失败: {}", e);
        // 尝试恢复本地状态文件
        if let Ok(_) = tokio::fs::copy(&local_state_temp, &local_state_path).await {
            let _ = tokio::fs::remove_file(&local_state_temp).await;
        }
        let _ = tokio::fs::remove_file(&remote_state_temp).await;
        return Err(anyhow!("无法更新远程状态文件: {}", e));
    }

    Ok(())
}

/// 加载同步状态
pub async fn load_sync_state() -> Result<(Option<FileSystemState>, Option<FileSystemState>)> {
    let state_dir = with_app_path(|app| app.config_dir().join("sync_state"));
    let local_state_path = state_dir.join("local_state.json");
    let remote_state_path = state_dir.join("remote_state.json");

    // 如果状态目录不存在，返回空状态
    if !state_dir.exists() {
        return Ok((None, None));
    }

    // 尝试加载本地状态
    let local_state = if local_state_path.exists() {
        match load_state(&local_state_path).await {
            Ok(state) => {
                Some(state)
            }
            Err(e) => {
                warn!("加载本地状态失败: {}", e);
                // 如果文件损坏，尝试备份并删除
                let backup_path = local_state_path.with_extension("json.bak");
                if let Ok(_) = tokio::fs::copy(&local_state_path, &backup_path).await {
                    let _ = tokio::fs::remove_file(&local_state_path).await;
                }
                None
            }
        }
    } else {
        None
    };

    // 尝试加载远程状态
    let remote_state = if remote_state_path.exists() {
        match load_state(&remote_state_path).await {
            Ok(state) => {
                Some(state)
            }
            Err(e) => {
                warn!("加载远程状态失败: {}", e);
                // 如果文件损坏，尝试备份并删除
                let backup_path = remote_state_path.with_extension("json.bak");
                if let Ok(_) = tokio::fs::copy(&remote_state_path, &backup_path).await {
                    let _ = tokio::fs::remove_file(&remote_state_path).await;
                }
                None
            }
        }
    } else {
        None
    };

    if local_state.is_none() && remote_state.is_none() {
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

    // 首先处理目录操作，确保目录结构存在
    diff.entries
        .iter()
        .for_each(|entry| match entry.entry_type {
            EntryType::Directory => do_when_dir(session, entry),
            EntryType::File => do_when_file(session, entry, conflict_strategy),
        });

    Ok(())
}

// 辅助函数
fn do_when_dir(session: &mut SyncSession, entry: &DiffEntry) {
    // 处理目录的同步操作
    match entry.diff_type {
        DiffType::Added => {
            if entry.local_state.is_some() && entry.remote_state.is_none() {
                session.add_operation(SyncOperation::create_remote_directory(entry));
            } else if entry.remote_state.is_some() && entry.local_state.is_none() {
                session.add_operation(SyncOperation::create_local_directory(entry));
            }
        }
        DiffType::Deleted => {
            // 临时禁用删除操作
        }
        _ => {} // 对于目录，其他差异类型不需要特殊处理
    }
}

fn do_when_file(session: &mut SyncSession, entry: &DiffEntry, conflict_strategy: ConflictStrategy) {
    match entry.diff_type {
        DiffType::Added => match (&entry.local_state, &entry.remote_state) {
            (Some(_), None) => {
                session.add_operation(SyncOperation::upload(entry));
            }
            (None, Some(_)) => {
                session.add_operation(SyncOperation::download(entry));
            }
            _ => {}
        },
        DiffType::Deleted => {
            // 检测到删除文件，但需要检查这个文件是真的被删除了还是被错误分类了

            // 如果远程存在但本地不存在，这应该是一个新的远程文件，需要下载而不是删除
            match (&entry.local_state, &entry.remote_state) {
                (None, Some(_)) => {
                    session.add_operation(SyncOperation::download(entry));
                }
                (Some(_), None) => {
                    session.add_operation(SyncOperation::upload(entry));
                }
                _ => {
                    warn!("无法确定删除操作的正确性，跳过: {}", entry.path.display());
                }
            }
        }
        DiffType::Modified => {
            // 文件被修改，根据冲突策略决定操作
            match conflict_strategy {
                ConflictStrategy::PreferLocal => {
                    // 优先使用本地版本，上传
                    warn!("计划上传本地修改的文件: {}", entry.path.display());
                    session.add_operation(SyncOperation::upload(entry));
                }
                ConflictStrategy::PreferRemote => {
                    // 优先使用远程版本，下载
                    warn!("计划下载远程修改的文件: {}", entry.path.display());
                    session.add_operation(SyncOperation::download(entry));
                }
                ConflictStrategy::KeepBoth => {
                    let path = Path::new(&entry.path);
                    let timestamp = Utc::now().timestamp();

                    let stem = format!(
                        "{}_remote_{}",
                        path.file_stem().and_then(|s| s.to_str()).unwrap_or(""),
                        timestamp
                    );

                    let mut path_buf = path.with_file_name(stem);
                    if let Some(ext) = path.extension() {
                        path_buf.set_extension(ext);
                    }

                    warn!(
                        "计划下载远程版本并重命名: {} -> {}",
                        entry.path.display(),
                        path_buf.display()
                    );
                    // 用新路径下载远程文件
                    session.add_operation(SyncOperation::download(entry));
                }
                ConflictStrategy::Skip => {
                    // 跳过冲突文件
                    warn!("跳过冲突文件: {}", entry.path.display());
                    session.add_operation(SyncOperation::skip(entry));
                }
            }
        }
        _ => {} // 其他情况不需要处理
    }
}

// 辅助函数
fn path_resolve(webdav_path: &Path, webdav_entry: &EntryState, remote_state: &mut FileSystemState) {
    let path_processor = PathResolver {};

    let Some(normalized_path) = path_processor.extract_relative_path(webdav_path) else {
        warn!(
            "extract_relative_path 返回 None，跳过: '{}'",
            webdav_path.display()
        );
        return;
    };

    if normalized_path.as_os_str().is_empty() {
        return;
    }

    let normalized_str = normalized_path.to_string_lossy();
    if normalized_str.contains("webdav") {
        error!(
            "路径处理失败,仍包含webdav信息: '{}'",
            normalized_path.display()
        );
        // 函数返回 ()，所以我们只记录错误并返回，而不是返回 Err
        return;
    }

    // 转换为相对路径用于内部存储（移除前导斜杠）
    let relative_path = PathBuf::from(normalized_str.trim_start_matches('/'));

    if !relative_path.contains("ToDoPulse") && webdav_path.contains("ToDoPulse") {
        error!("严重错误:ToDoPulse路径部分被意外移除！");
        error!("  原始路径: '{}'", webdav_path.display());
        error!("  标准化路径: '{}'", normalized_path.display());
        error!("  最终路径: '{}'", relative_path.display());
        // 同样，只记录错误并返回
        return;
    }

    if webdav_entry.is_file() {
        if let Some(size) = webdav_entry.size {
            let entry = EntryState::new_file(relative_path, webdav_entry.modified, size);
            remote_state.add_entry(entry);
        }
    } else {
        let entry = EntryState::new_directory(relative_path, webdav_entry.modified);
        remote_state.add_entry(entry);
    }

}
