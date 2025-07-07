use anyhow::{Result, anyhow};
use chrono::Utc;
use log::{debug, error, info, warn};
use std::path::Path;
use urlencoding;

use crate::webdav::WebDav;
use crate::diff::{DiffConfig, compare_states};
use crate::model::{
    ConflictStrategy, DiffResult, DiffType, EntryState, FileSystemState, SyncOperation,
    SyncOperationStatus, SyncOperationType, SyncSession, SyncSessionStatus,
};
use crate::state::{StateCollectionConfig, collect_local_state, load_state, save_state};

/// 路径处理工具
struct PathProcessor {}

impl PathProcessor {
    /// 从完整的WebDAV路径提取标准化路径
    /// 输入: "/webdav/webdav/ToDoPulse/5555.md"
    /// 输出: "/ToDoPulse/5555.md"
    fn extract_relative_path(&self, webdav_url: &str) -> Option<String> {
        debug!("=== extract_relative_path 开始 ===");
        debug!("输入路径: '{}'", webdav_url);

        // URL解码
        let decoded = match urlencoding::decode(webdav_url) {
            Ok(d) => d.into_owned(),
            Err(e) => {
                warn!("URL解码失败: {} - {}", webdav_url, e);
                return None;
            }
        };
        debug!("URL解码后: '{}'", decoded);

        // 固定移除 "/webdav/webdav" 前缀
        const WEBDAV_PREFIX: &str = "/webdav/webdav";

        if decoded.starts_with(WEBDAV_PREFIX) {
            let after_prefix = &decoded[WEBDAV_PREFIX.len()..];

            // 如果结果为空或只是斜杠，表示这是根目录
            if after_prefix.is_empty() || after_prefix == "/" {
                debug!("检测到根目录，返回空字符串");
                return Some(String::new());
            }

            // 确保以斜杠开头的标准格式
            let normalized = format!("/{}", after_prefix.trim_start_matches('/'));

            debug!("最终标准化结果: '{}'", normalized);
            Some(normalized)
        } else {
            warn!("路径不以webdav前缀开头: '{}'", decoded);
            None
        }
    }
}

/// 将配置应用到会话
pub fn apply_config_to_session(session: &mut SyncSession) -> Result<()> {
    let webdav_config = WebDav::load()?;

    // 检查WebDAV是否启用
    if !webdav_config.enabled {
        return Err(anyhow!("WebDAV同步未启用"));
    }

    // 更新会话的远程目录
    session.remote_dir = webdav_config.remote_dir.clone().into();

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

    let webdav_config = WebDav::load()?;
    if !webdav_config.enabled {
        return Err(anyhow!("WebDAV同步未启用"));
    }

    let client = crate::create_client(
        &webdav_config.host,
        &webdav_config.username,
        &webdav_config.password,
    )
    .await?;

    // 创建路径处理器
    let path_processor = PathProcessor {};

    // 收集本地状态
    info!("收集本地文件系统状态: {}", session.local_dir.display());
    let state_config = StateCollectionConfig::default();
    let local_state = collect_local_state(&session.local_dir, &state_config).await?;

    // 收集远程状态
    info!("收集远程文件系统状态: {}", session.remote_dir.display());
    let webdav_remote_state = crate::collect_remote_state(&client, &session.remote_dir).await?;

    // 添加详细的原始路径调试
    info!("WebDAV 原始收集结果:");
    for (original_path, entry) in &webdav_remote_state.entries {
        info!(
            "  原始路径: '{}', 类型: {}",
            original_path.display(),
            if entry.is_file() { "文件" } else { "目录" }
        );
    }

    // 转换远程状态
    let mut remote_state = FileSystemState::new();
    remote_state.collection_time = webdav_remote_state.collection_time;

    info!(
        "WebDAV 收集到 {} 个原始条目",
        webdav_remote_state.entries.len()
    );

    for (webdav_path, webdav_entry) in &webdav_remote_state.entries {
        debug!("=== 开始处理路径 ===");
        debug!("原始WebDAV路径: '{}'", webdav_path.display());

        if let Some(normalized_path) = path_processor.extract_relative_path(webdav_path) {
            debug!("extract_relative_path 返回: '{}'", normalized_path);

            if normalized_path.is_empty() {
                debug!("路径为空，跳过根目录: '{}'", webdav_path.display());
                continue;
            }

            info!(
                "路径处理: '{}' -> '{}'",
                webdav_path.display(),
                normalized_path.display()
            );

            // 验证：确保路径不再包含webdav信息
            if normalized_path.contains("webdav") {
                error!("路径处理失败，仍包含webdav信息: '{}'", normalized_path);
                return Err(anyhow!("路径处理错误: '{}'", normalized_path));
            }

            // 转换为相对路径用于内部存储（移除前导斜杠）
            let relative_path = normalized_path.trim_start_matches('/');
            debug!("内部存储路径: '{}'", relative_path);

            // 关键验证：检查是否意外移除了重要路径部分
            if !relative_path.contains("ToDoPulse") && webdav_path.contains("ToDoPulse") {
                error!("严重错误：ToDoPulse路径部分被意外移除！");
                error!("原始路径: '{}'", webdav_path.display());
                error!("标准化路径: '{}'", normalized_path.display());
                error!("最终路径: '{}'", relative_path.display());
                return Err(anyhow!("路径处理错误：ToDoPulse部分丢失"));
            }

            if webdav_entry.is_file() {
                let entry = EntryState::new_file(
                    relative_path,
                    webdav_entry.modified,
                    webdav_entry.size,
                );
                remote_state.add_entry(entry);
            } else {
                let entry =
                    EntryState::new_directory(relative_path.to_string(), webdav_entry.modified);
                remote_state.add_entry(entry);
            }
        } else {
            warn!("extract_relative_path 返回 None: '{}'", webdav_path.display());
        }
        debug!("=== 路径处理完成 ===");
    }

    info!(
        "状态收集完成 - 本地: {} 条目, 远程: {} 条目",
        local_state.entries.len(),
        remote_state.entries.len()
    );

    // 最终验证
    for (path, _) in &remote_state.entries {
        if path.contains("webdav") {
            return Err(anyhow!("状态收集失败，路径包含webdav信息: '{}'", path));
        }
    }

    Ok((local_state, remote_state))
}

/// 执行同步操作
pub async fn execute_sync_operations(
    mut session: SyncSession,
    webdav_config: WebDav,
) -> Result<SyncSession> {
    session.status = SyncSessionStatus::Executing;
    info!("执行同步操作");

    if !webdav_config.enabled {
        return Err(anyhow!("WebDAV同步未启用"));
    }

    let client = crate::create_client(
        &webdav_config.host,
        &webdav_config.username,
        &webdav_config.password,
    )
    .await?;

    // 创建路径处理器
    let path_processor = PathProcessor {};

    // 确保远程目录存在
    crate::ensure_remote_dir_exists(&client, &session.remote_dir).await?;

    for operation in &mut session.operations {
        if operation.status != SyncOperationStatus::Pending {
            continue;
        }

        operation.status = SyncOperationStatus::InProgress;

        // operation.path 应该是相对路径，如 "ToDoPulse/dffff/dffff.md"
        let relative_path = operation.path.trim_start_matches('/');

        // 从相对路径中移除 ToDoPulse 前缀来构建本地路径
        let local_relative_path = relative_path
            .strip_prefix("ToDoPulse/")
            .unwrap_or(relative_path);

        match operation.operation_type {
            SyncOperationType::Download => {
                info!("下载文件: {}", relative_path);

                let local_path = session.local_dir.join(local_relative_path);
                let remote_path = format!("/{}", relative_path.trim_start_matches('/'));

                info!(
                    "下载 - 本地: {}, 远程: {}",
                    local_path.display(),
                    remote_path
                );

                // 确保父目录存在
                if let Some(parent) = local_path.parent() {
                    if let Err(e) = tokio::fs::create_dir_all(parent).await {
                        operation.status = SyncOperationStatus::Failed;
                        operation.error = Some(format!("创建父目录失败: {}", e));
                        continue;
                    }
                }

                match crate::download_file(&client, &remote_path, &local_path).await {
                    Ok(_) => {
                        operation.status = SyncOperationStatus::Completed;
                        info!("下载完成: {}", relative_path);
                    }
                    Err(e) => {
                        operation.status = SyncOperationStatus::Failed;
                        operation.error = Some(format!("下载失败: {}", e));
                        error!("下载失败 {}: {}", relative_path, e);
                    }
                }
            }
            SyncOperationType::Upload => {
                info!("上传文件: {}", relative_path);

                let local_path = session.local_dir.join(local_relative_path);
                let remote_path = format!("/{}", relative_path.trim_start_matches('/'));

                info!(
                    "上传 - 本地: {}, 远程: {}",
                    local_path.display(),
                    remote_path
                );

                if !local_path.exists() {
                    operation.status = SyncOperationStatus::Failed;
                    operation.error = Some(format!("本地文件不存在: {}", local_path.display()));
                    continue;
                }

                match upload_file(&client, &local_path, &remote_path).await {
                    Ok(_) => {
                        operation.status = SyncOperationStatus::Completed;
                        info!("上传完成: {}", relative_path);
                    }
                    Err(e) => {
                        operation.status = SyncOperationStatus::Failed;
                        operation.error = Some(format!("上传失败: {}", e));
                        error!("上传失败 {}: {}", relative_path, e);
                    }
                }
            }
            SyncOperationType::CreateLocalDirectory => {
                info!("创建本地目录: {}", relative_path);

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
                info!("创建远程目录: {}", relative_path);

                let remote_path = format!("/{}", relative_path.trim_start_matches('/'));

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

    let stats = session.get_stats();
    info!(
        "同步完成: 总操作 {}, 成功 {}, 失败 {}, 跳过 {}, 用时 {}秒",
        stats.total, stats.completed, stats.failed, stats.skipped, stats.duration_seconds
    );

    Ok(session)
}

/// 执行完整的同步流程
pub async fn perform_sync() -> Result<SyncSession> {
    // 创建同步会话
    let mut session = create_sync_session()?;

    // 尝试加载上一次的状态
    let states = load_sync_state()
        .await
        .map_err(|e| {
            warn!("无法加载上一次的同步状态: {}", e);
            info!("将进行完整同步");
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
            info!("正在与上次同步状态比较差异");

            // 检查本地变化
            let local_changes = compare_states(&saved_local, &local_state, &diff_config)?;

            // 检查远程变化
            let remote_changes = compare_states(&saved_remote, &remote_state, &diff_config)?;

            info!(
                "检测到本地变化: {} 项, 远程变化: {} 项",
                local_changes.entries.len(),
                remote_changes.entries.len()
            );

            // 合并差异
            let mut combined_diff = local_changes;
            combined_diff
                .entries
                .extend(remote_changes.entries.iter().cloned());
            combined_diff
        } else {
            // 没有历史状态，直接比较当前状态
            compare_states(&local_state, &remote_state, &diff_config)?
        };

    // 计划同步操作
    plan_sync_operations(&mut session, &diff, ConflictStrategy::PreferLocal)?;

    // 如果没有操作需要执行，直接完成
    if session.operations.is_empty() {
        info!("没有需要同步的内容");
        session.status = SyncSessionStatus::Completed;
        session.end_time = Some(Utc::now());
        return Ok(session);
    }

    // 执行同步操作
    execute_sync_operations(session).await
}

/// 保存和加载同步状态
#[allow(dead_code)]
pub async fn save_sync_state(
    local_state: &FileSystemState,
    remote_state: &FileSystemState,
) -> Result<()> {
    info!("开始保存同步状态");

    // 创建状态目录
    let state_dir = AppPaths::config_dir().join("sync_state");
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

    info!("同步状态保存完成");
    Ok(())
}

/// 加载同步状态
pub async fn load_sync_state() -> Result<(Option<FileSystemState>, Option<FileSystemState>)> {
    info!("开始加载同步状态");
    let state_dir = AppPaths::config_dir().join("sync_state");
    let local_state_path = state_dir.join("local_state.json");
    let remote_state_path = state_dir.join("remote_state.json");

    // 如果状态目录不存在，返回空状态
    if !state_dir.exists() {
        info!("同步状态目录不存在，返回空状态");
        return Ok((None, None));
    }

    // 尝试加载本地状态
    let local_state = if local_state_path.exists() {
        match load_state(&local_state_path).await {
            Ok(state) => {
                info!("成功加载本地状态，包含 {} 个条目", state.entry_count());
                Some(state)
            }
            Err(e) => {
                warn!("加载本地状态失败: {}", e);
                // 如果文件损坏，尝试备份并删除
                let backup_path = local_state_path.with_extension("json.bak");
                if let Ok(_) = tokio::fs::copy(&local_state_path, &backup_path).await {
                    info!("已创建损坏的本地状态文件备份");
                    let _ = tokio::fs::remove_file(&local_state_path).await;
                }
                None
            }
        }
    } else {
        debug!("本地状态文件不存在");
        None
    };

    // 尝试加载远程状态
    let remote_state = if remote_state_path.exists() {
        match load_state(&remote_state_path).await {
            Ok(state) => {
                info!("成功加载远程状态，包含 {} 个条目", state.entry_count());
                Some(state)
            }
            Err(e) => {
                warn!("加载远程状态失败: {}", e);
                // 如果文件损坏，尝试备份并删除
                let backup_path = remote_state_path.with_extension("json.bak");
                if let Ok(_) = tokio::fs::copy(&remote_state_path, &backup_path).await {
                    info!("已创建损坏的远程状态文件备份");
                    let _ = tokio::fs::remove_file(&remote_state_path).await;
                }
                None
            }
        }
    } else {
        debug!("远程状态文件不存在");
        None
    };

    if local_state.is_none() && remote_state.is_none() {
        info!("未能加载任何同步状态");
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

    // 调试：输出所有差异条目的详细信息
    info!("差异分析结果：");
    for entry in &diff.entries {
        info!("差异分析结果:{}", entry);
    }

    // 首先处理目录操作，确保目录结构存在
    for entry in &diff.entries {
        if entry.is_directory() {
            match entry.diff_type {
                DiffType::Added => {
                    // 本地添加的目录，需要在远程创建
                    if entry.local_state.is_some() && entry.remote_state.is_none() {
                        info!("计划创建远程目录: {}", entry.path.display());
                        session.add_operation(SyncOperation {
                            operation_type: SyncOperationType::CreateRemoteDirectory,
                            path: entry.path.clone(),
                            entry_type: entry.entry_type.clone(),
                            status: SyncOperationStatus::Pending,
                            error: None,
                        });
                    }
                    // 远程添加的目录，需要在本地创建
                    else if entry.remote_state.is_some() && entry.local_state.is_none() {
                        info!("计划创建本地目录: {}", entry.path.display());
                        session.add_operation(SyncOperation {
                            operation_type: SyncOperationType::CreateLocalDirectory,
                            path: entry.path.clone(),
                            entry_type: entry.entry_type.clone(),
                            status: SyncOperationStatus::Pending,
                            error: None,
                        });
                    }
                }
                DiffType::Deleted => {
                    // 临时禁用删除操作
                    info!("跳过删除目录操作: {})", entry);
                }
                _ => {} // 对于目录，其他差异类型不需要特殊处理
            }
        }
    }

    // 然后处理文件操作
    for entry in &diff.entries {
        if entry.is_file() {
            match entry.diff_type {
                DiffType::Added => {
                    // 本地添加的文件，需要上传
                    if entry.local_state.is_some() && entry.remote_state.is_none() {
                        info!("计划上传本地新文件: {}", entry.path.display());
                        session.add_operation(SyncOperation {
                            operation_type: SyncOperationType::Upload,
                            path: entry.path.clone(),
                            entry_type: entry.entry_type.clone(),
                            status: SyncOperationStatus::Pending,
                            error: None,
                        });
                    }
                    // 远程添加的文件，需要下载
                    else if entry.remote_state.is_some() && entry.local_state.is_none() {
                        info!("计划下载远程新文件: {}", entry.path.display());
                        session.add_operation(SyncOperation {
                            operation_type: SyncOperationType::Download,
                            path: entry.path.clone(),
                            entry_type: entry.entry_type.clone(),
                            status: SyncOperationStatus::Pending,
                            error: None,
                        });
                    }
                }
                DiffType::Deleted => {
                    // 检测到删除文件，但需要检查这个文件是真的被删除了还是被错误分类了
                    info!("检测到删除文件: {})", entry);

                    // 如果远程存在但本地不存在，这应该是一个新的远程文件，需要下载而不是删除
                    if entry.remote_state.is_some() && entry.local_state.is_none() {
                        warn!(
                            "差异分析错误：文件 {} 被错误地标记为删除，实际应该下载",
                            entry.path.display()
                        );
                        info!("修正：计划下载远程新文件: {}", entry.path.display());
                        session.add_operation(SyncOperation {
                            operation_type: SyncOperationType::Download,
                            path: entry.path.clone(),
                            entry_type: entry.entry_type.clone(),
                            status: SyncOperationStatus::Pending,
                            error: None,
                        });
                    }
                    // 如果本地存在但远程不存在，这是一个本地文件需要上传
                    else if entry.local_state.is_some() && entry.remote_state.is_none() {
                        warn!(
                            "差异分析错误：文件 {} 被错误地标记为删除，实际应该上传",
                            entry.path.display()
                        );
                        info!("修正：计划上传本地新文件: {}", entry.path.display());
                        session.add_operation(SyncOperation {
                            operation_type: SyncOperationType::Upload,
                            path: entry.path.clone(),
                            entry_type: entry.entry_type.clone(),
                            status: SyncOperationStatus::Pending,
                            error: None,
                        });
                    } else {
                        info!("跳过删除文件操作: {}", entry.path.display());
                    }
                }
                DiffType::Modified => {
                    // 文件被修改，根据冲突策略决定操作
                    match conflict_strategy {
                        ConflictStrategy::PreferLocal => {
                            // 优先使用本地版本，上传
                            info!("计划上传本地修改的文件: {}", entry.path.display());
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
                            info!("计划下载远程修改的文件: {}", entry.path.display());
                            session.add_operation(SyncOperation {
                                operation_type: SyncOperationType::Download,
                                path: entry.path.clone(),
                                entry_type: entry.entry_type.clone(),
                                status: SyncOperationStatus::Pending,
                                error: None,
                            });
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

                            info!(
                                "计划下载远程版本并重命名: {} -> {}",
                                entry.path.display(),
                                path_buf.display()
                            );
                            // 用新路径下载远程文件
                            session.add_operation(SyncOperation {
                                operation_type: SyncOperationType::Download,
                                path: path_buf,
                                entry_type: entry.entry_type.clone(),
                                status: SyncOperationStatus::Pending,
                                error: None,
                            });
                        }
                        ConflictStrategy::Skip => {
                            // 跳过冲突文件
                            info!("跳过冲突文件: {}", entry.path.display());
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
