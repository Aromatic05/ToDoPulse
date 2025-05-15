use crate::function::sync::model::{EntryState, FileSystemState};
use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use log::{debug, error, info, warn};
use std::path::Path;
use tokio::fs;

/// 状态收集配置
pub struct StateCollectionConfig {
    /// 是否计算内容哈希（可能导致性能下降）
    pub compute_hash: bool,
    /// 要排除的文件或目录模式
    pub exclusion_patterns: Vec<String>,
    /// 最大递归深度（None表示无限制）
    pub max_depth: Option<usize>,
}

impl Default for StateCollectionConfig {
    fn default() -> Self {
        Self {
            compute_hash: false,
            exclusion_patterns: vec![
                // 常见临时文件和隐藏文件
                ".DS_Store".to_string(),
                "Thumbs.db".to_string(),
                "*.tmp".to_string(),
                "*.temp".to_string(),
                "~*".to_string(),
                // 常见VCS目录
                ".git".to_string(),
                ".svn".to_string(),
            ],
            max_depth: None,
        }
    }
}

/// 判断文件是否应该被排除
pub(crate) fn should_exclude(path: &Path, patterns: &[String]) -> bool {
    let file_name = path
        .file_name()
        .map(|n| n.to_string_lossy())
        .unwrap_or_default();

    for pattern in patterns {
        // 简单的通配符匹配
        if pattern.starts_with("*") && pattern.len() > 1 {
            let suffix = &pattern[1..];
            if file_name.ends_with(suffix) {
                return true;
            }
        } else if pattern.ends_with("*") && pattern.len() > 1 {
            let prefix = &pattern[..pattern.len() - 1];
            if file_name.starts_with(prefix) {
                return true;
            }
        } else if file_name.to_string() == *pattern {
            return true;
        }
    }

    false
}

/// 从系统时间获取UTC时间，使用chrono的方法直接转换
pub(crate) fn system_time_to_datetime(time: std::time::SystemTime) -> Option<DateTime<Utc>> {
    // 使用chrono提供的From trait实现
    let datetime: DateTime<Utc> = time.into();
    Some(datetime)
}

/// 计算文件内容的简单哈希值
pub(crate) async fn compute_file_hash(path: &Path) -> Result<String> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    use tokio::io::AsyncReadExt;

    // 使用Rust标准库的哈希功能
    let mut file = fs::File::open(path).await?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).await?;

    let mut hasher = DefaultHasher::new();
    buffer.hash(&mut hasher);
    let hash = hasher.finish();

    Ok(format!("{:x}", hash))
}

/// 收集本地文件系统状态
pub async fn collect_local_state(
    root_dir: &Path,
    config: &StateCollectionConfig,
) -> Result<FileSystemState> {
    debug!("收集本地文件系统状态: {}", root_dir.display());

    if !root_dir.exists() {
        return Err(anyhow!("目录不存在: {}", root_dir.display()));
    }
    if !root_dir.is_dir() {
        return Err(anyhow!("路径不是目录: {}", root_dir.display()));
    }

    let canonical_root = match fs::canonicalize(root_dir).await {
        Ok(p) => p,
        Err(e) => {
            error!("无法获取根目录的绝对路径 {}: {}", root_dir.display(), e);
            return Err(anyhow!("无法获取根目录的绝对路径: {}", e));
        }
    };

    let mut state = FileSystemState::new();

    // 递归遍历目录收集状态的辅助函数
    fn scan_directory<'a>(
        canonical_root_path: &'a Path,
        current_dir_abs_path: &'a Path,
        state: &'a mut FileSystemState,
        config: &'a StateCollectionConfig,
        current_depth: usize,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + 'a>> {
        Box::pin(async move {
            let should_process_children = match config.max_depth {
                Some(max_depth) => current_depth < max_depth,
                None => true,
            };

            let mut entries = match fs::read_dir(current_dir_abs_path).await {
                Ok(entries) => entries,
                Err(e) => {
                    warn!("无法读取目录 {}: {}", current_dir_abs_path.display(), e);
                    return Ok(());
                }
            };

            while let Some(entry) = entries.next_entry().await? {
                // 此处 'entry' 的类型是 tokio::fs::DirEntry

                let abs_path_from_entry = entry.path();
                let entry_abs_path = if abs_path_from_entry.is_absolute() {
                    abs_path_from_entry.clone()
                } else {
                    current_dir_abs_path.join(entry.file_name())
                };

                let canonical_entry_abs_path = match fs::canonicalize(&entry_abs_path).await {
                    Ok(p) => p,
                    Err(e) => {
                        warn!(
                            "无法规范化路径 {} (原始路径 {}): {}",
                            entry_abs_path.display(),
                            abs_path_from_entry.display(),
                            e
                        );
                        continue; // 跳过无法规范化的条目
                    }
                };

                if should_exclude(&canonical_entry_abs_path, &config.exclusion_patterns) {
                    debug!("排除路径: {}", canonical_entry_abs_path.display());
                    continue;
                }

                let rel_path_str = match canonical_entry_abs_path.strip_prefix(canonical_root_path)
                {
                    Ok(r) => format!("/{}", r.to_string_lossy().replace('\\', "/")),
                    Err(_) => {
                        error!(
                            "无法创建相对路径 for {} from root {}",
                            canonical_entry_abs_path.display(),
                            canonical_root_path.display()
                        );
                        continue;
                    }
                };

                debug!(
                    "处理条目: {} -> 相对路径: {}",
                    canonical_entry_abs_path.display(),
                    rel_path_str
                );

                let metadata = match entry.metadata().await {
                    // 可以直接用 entry.metadata() 避免再次 stat
                    Ok(meta) => meta,
                    Err(e) => {
                        warn!(
                            "无法获取元数据 {}: {}",
                            canonical_entry_abs_path.display(),
                            e
                        );
                        continue;
                    }
                };
                let modified = metadata.modified().ok().and_then(system_time_to_datetime);

                if metadata.is_dir() {
                    state.add_entry(EntryState::new_directory(rel_path_str.clone(), modified));
                    debug!("添加目录条目: {}", rel_path_str);

                    if should_process_children {
                        scan_directory(
                            canonical_root_path,
                            &canonical_entry_abs_path,
                            state,
                            config,
                            current_depth + 1,
                        )
                        .await?;
                    } else {
                        debug!(
                            "跳过目录内容处理 (已达最大深度或配置不允许): {}",
                            canonical_entry_abs_path.display()
                        );
                    }
                } else if metadata.is_file() {
                    let size = Some(metadata.len());
                    let mut entry_state =
                        EntryState::new_file(rel_path_str.clone(), modified, size);

                    if config.compute_hash {
                        match compute_file_hash(&canonical_entry_abs_path).await {
                            Ok(hash) => entry_state = entry_state.with_hash(hash),
                            Err(e) => warn!(
                                "计算文件哈希失败 {}: {}",
                                canonical_entry_abs_path.display(),
                                e
                            ),
                        }
                    }
                    state.add_entry(entry_state);
                    debug!("添加文件条目: {}", rel_path_str);
                }
            }
            Ok(())
        })
    }

    scan_directory(&canonical_root, &canonical_root, &mut state, config, 0).await?;

    info!(
        "本地状态收集完成，共 {} 个条目 ({} 文件, {} 目录)",
        state.entry_count(),
        state.file_count(),
        state.directory_count()
    );

    Ok(state)
}

/// 保存文件系统状态到磁盘
#[allow(dead_code)]
pub async fn save_state(state: &FileSystemState, path: &Path) -> Result<()> {
    let json = serde_json::to_string_pretty(state)?;
    fs::write(path, json).await?;
    debug!("状态已保存到: {}", path.display());
    Ok(())
}

/// 从磁盘加载文件系统状态
#[allow(dead_code)]
pub async fn load_state(path: &Path) -> Result<FileSystemState> {
    let json = fs::read_to_string(path).await?;
    let state: FileSystemState = serde_json::from_str(&json)?;
    debug!(
        "从 {} 加载了状态，共 {} 个条目",
        path.display(),
        state.entry_count()
    );
    Ok(state)
}