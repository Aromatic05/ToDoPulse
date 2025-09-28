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

        // 如果 rel_path 为空，表示被排除或是根占位，跳过
        if rel_path.as_os_str().is_empty() {
            continue;
        }

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
            // 递归到真正的子目录（canonical_entry_abs），而不是重入当前目录，避免无限递归
            scan_directory(
                canonical_root,
                &canonical_entry_abs,
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

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use chrono::{DateTime, Utc};
    use std::io::Write;
    use std::path::{Path, PathBuf};
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

        let p1 = PathBuf::from("ToDoPulse/file1.txt");
        let p2 = PathBuf::from("ToDoPulse/subdir/file2.txt");
        let p3 = PathBuf::from("ToDoPulse/subdir");

        // 检查文件项是否正确
        assert!(
            state.entries.get(&p1).map_or(false, |e| e.is_file()),
            "file1.txt 应该是文件"
        );
        assert!(
            state.entries.get(&p2).map_or(false, |e| e.is_file()),
            "subdir/file2.txt 应该是文件"
        );
        assert!(
            state.entries.get(&p3).map_or(false, |e| e.is_directory()),
            "subdir 应该是目录"
        );

        // 现在断言文件数量
        assert_eq!(state.file_count(), 2, "应该有两个文件");
        assert_eq!(state.directory_count(), 1); // subdir

        // 验证路径正确性
        assert!(state.entries.contains_key(&p1));
        assert!(state.entries.contains_key(&p3));
        assert!(state.entries.contains_key(&p2));

        // 验证被排除的文件不在结果中
        assert!(
            !state
                .entries
                .contains_key(&PathBuf::from("ToDoPulse/.DS_Store"))
        );

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
        let now = Utc::now();

        let file_entry = EntryState::new_file("/test/file.txt".into(), now, 1024)
            .with_hash("abcdef1234567890".to_string());

        let dir_entry = EntryState::new_directory("/test/dir".into(), now);

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

        let p_file = PathBuf::from("/test/file.txt");
        let p_dir = PathBuf::from("/test/dir");

        // 验证具体条目
        assert!(loaded_state.entries.contains_key(&p_file));
        assert!(loaded_state.entries.contains_key(&p_dir));

        let loaded_file_entry = loaded_state.entries.get(&p_file).unwrap();
        assert_eq!(loaded_file_entry.path, p_file);
        assert_eq!(loaded_file_entry.size, Some(1024));
        assert_eq!(
            loaded_file_entry.content_hash,
            Some("abcdef1234567890".to_string())
        );
        assert!(loaded_file_entry.is_file());

        let loaded_dir_entry = loaded_state.entries.get(&p_dir).unwrap();
        assert_eq!(loaded_dir_entry.path, p_dir);
        assert!(loaded_dir_entry.is_directory());

        Ok(())
    }

    #[tokio::test]
    async fn test_system_time_to_datetime() {
        // 创建一个SystemTime
        let now_sys = std::time::SystemTime::now();

        // 转换为DateTime
        let now_dt: DateTime<Utc> = now_sys.into();

        // 确保它是一个有效的DateTime
        assert!(now_dt.timestamp() > 0);

        // 由于精度限制，直接比较可能不准确，但应该非常接近
        let now_ts = now_sys
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let dt_ts = now_dt.timestamp() as u64;

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

        let p_file = PathBuf::from("ToDoPulse/file_with_hash.txt");

        // 验证文件哈希
        let file_entry = state.entries.get(&p_file).expect("文件应该存在");
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
}
