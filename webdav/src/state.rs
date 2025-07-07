use crate::model::{EntryState, FileSystemState};
use anyhow::{Result, anyhow};
use async_recursion::async_recursion;
use chrono::{DateTime, Utc};
use log::{debug, error, info};
use std::path::{Path, PathBuf};
use tokio::fs::{self, DirEntry};

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

// 递归遍历目录收集状态的辅助函数
#[async_recursion]
async fn scan_directory(
    canonical_root: &Path,
    current_dir_abs: &Path,
    state: &mut FileSystemState,
    config: &StateCollectionConfig,
    current_depth: usize,
) -> Result<()> {
    let mut entries = fs::read_dir(current_dir_abs).await.map_err(|e| {
        error!("无法读取目录 {}: {}", current_dir_abs.display(), e);
        anyhow!("无法读取目录: {}", e)
    })?;

    while let Some(entry) = entries.next_entry().await? {
        let (rel_path, canonical_entry_abs) = rel_path(&entry, config, canonical_root).await?;

        let metadata = entry
            .metadata()
            .await
            .map_err(|e| anyhow!("无法获取条目元数据: {}", e))?;

        let modified: DateTime<Utc> = metadata.modified().ok().map_or_else(
            || {
                error!("无法获取条目 {} 的修改时间", canonical_entry_abs.display());
                Utc::now()
            },
            |t| DateTime::from(t),
        );

        if metadata.is_dir() {
            state.add_entry(EntryState::new_directory(rel_path, modified));

            let should_process_children = config.max_depth.map_or(true, |max| current_depth < max);
            if !should_process_children {
                debug!(
                    "跳过目录 {}，已达到最大深度 {}",
                    canonical_entry_abs.display(),
                    current_depth
                );
                continue;
            }
            scan_directory(
                canonical_root,
                current_dir_abs,
                state,
                config,
                current_depth + 1,
            )
            .await?;
        } else if metadata.is_file() {
            let size = metadata.len();
            let mut entry_state = EntryState::new_file(rel_path, modified, size);
            if config.compute_hash {
                entry_state = entry_state.with_hash(compute_file_hash(&canonical_entry_abs).await?);
            }
            state.add_entry(entry_state);
        }
    }
    Ok(())
}

// 辅助函数
async fn rel_path(
    entry: &DirEntry,
    config: &StateCollectionConfig,
    canonical_root: &Path,
) -> Result<(PathBuf, PathBuf)> {
    let entry_abs = entry.path();

    let canonical_entry_abs = fs::canonicalize(&entry_abs).await.map_err(|e| {
        error!("无法获取条目 {} 的绝对路径: {}", entry_abs.display(), e);
        anyhow!("无法获取条目绝对路径: {}", e)
    })?;

    if should_exclude(&canonical_entry_abs, &config.exclusion_patterns) {
        debug!("排除路径: {}", canonical_entry_abs.display());
        return Ok((PathBuf::new(), canonical_entry_abs));
    }

    let rel_path_str = canonical_entry_abs
        .strip_prefix(canonical_root)
        .ok()
        .map_or_else(
            || {
                error!(
                    "无法创建相对路径 for {} from root {}",
                    canonical_entry_abs.display(),
                    canonical_root.display()
                );
                "".into()
            },
            |r| {
                // 添加 ToDoPulse 前缀，确保与远程路径格式一致
                format!("ToDoPulse/{}", r.to_string_lossy().replace('\\', "/"))
            },
        );

    debug!(
        "处理条目: {} -> 相对路径: {}",
        canonical_entry_abs.display(),
        rel_path_str
    );

    Ok((PathBuf::from(&rel_path_str), canonical_entry_abs))
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
