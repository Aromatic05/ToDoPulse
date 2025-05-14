use anyhow::{Result, anyhow};
use log::{info, warn, error, debug};
use tokio::fs;
use std::path::{Path};
use chrono::{DateTime, Utc};
use crate::function::sync::model::{EntryState, FileSystemState};

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
fn should_exclude(path: &Path, patterns: &[String]) -> bool {
    let file_name = path.file_name()
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
fn system_time_to_datetime(time: std::time::SystemTime) -> Option<DateTime<Utc>> {
    // 使用chrono提供的From trait实现
    let datetime: DateTime<Utc> = time.into();
    Some(datetime)
}

/// 计算文件内容的简单哈希值
async fn compute_file_hash(path: &Path) -> Result<String> {
    use tokio::io::AsyncReadExt;
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
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
    config: &StateCollectionConfig
) -> Result<FileSystemState> {
    debug!("收集本地文件系统状态: {}", root_dir.display());
    
    // 检查目录是否存在
    if !root_dir.exists() {
        return Err(anyhow!("目录不存在: {}", root_dir.display()));
    }
    
    if !root_dir.is_dir() {
        return Err(anyhow!("路径不是目录: {}", root_dir.display()));
    }
    
    let mut state = FileSystemState::new();
    
    // 递归遍历目录收集状态
    fn scan_directory<'a>(
        root: &'a Path,
        dir: &'a Path,
        state: &'a mut FileSystemState,
        config: &'a StateCollectionConfig,
        current_depth: usize
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + 'a>> {
        Box::pin(async move {
            // 检查是否超过最大深度
            if let Some(max_depth) = config.max_depth {
                if current_depth > max_depth {
                    return Ok(());
                }
            }
            
            // 读取目录内容
            let mut entries = match fs::read_dir(dir).await {
                Ok(entries) => entries,
                Err(e) => {
                    error!("无法读取目录 {}: {}", dir.display(), e);
                    return Err(anyhow!("读取目录失败: {}", e));
                }
            };
            
            // 创建当前目录的相对路径
            let rel_path = if dir == root {
                "/".to_string()
            } else {
                let rel = dir.strip_prefix(root)
                    .map_err(|_| anyhow!("无法创建相对路径"))?;
                format!("/{}", rel.to_string_lossy().replace('\\', "/"))
            };
            
            // 添加目录本身到状态中
            let metadata = fs::metadata(dir).await?;
            let modified = metadata.modified().ok()
                .and_then(system_time_to_datetime);
            
            if rel_path != "/" {
                // 不添加根目录，因为它是同步的基准点
                state.add_entry(EntryState::new_directory(rel_path, modified));
            }
            
            // 处理目录中的每个条目
            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();
                
                // 检查是否应该排除
                if should_exclude(&path, &config.exclusion_patterns) {
                    debug!("排除路径: {}", path.display());
                    continue;
                }
                
                let metadata = match fs::metadata(&path).await {
                    Ok(meta) => meta,
                    Err(e) => {
                        warn!("无法获取元数据 {}: {}", path.display(), e);
                        continue;
                    }
                };
                
                let modified = metadata.modified().ok()
                    .and_then(system_time_to_datetime);
                
                // 创建相对路径
                let path_rel = path.strip_prefix(root)
                    .map_err(|_| anyhow!("无法创建相对路径"))?;
                let rel_path = format!("/{}", path_rel.to_string_lossy().replace('\\', "/"));
                
                if metadata.is_dir() {
                    // 递归处理子目录
                    scan_directory(
                        root,
                        &path,
                        state,
                        config,
                        current_depth + 1
                    ).await?;
                } else if metadata.is_file() {
                    // 处理文件
                    let size = Some(metadata.len());
                    let mut entry = EntryState::new_file(rel_path, modified, size);
                    
                    // 如果需要，计算文件哈希
                    if config.compute_hash {
                        match compute_file_hash(&path).await {
                            Ok(hash) => entry = entry.with_hash(hash),
                            Err(e) => warn!("计算文件哈希失败 {}: {}", path.display(), e),
                        }
                    }
                    
                    state.add_entry(entry);
                }
                // 忽略其他类型的文件系统条目
            }
            
            Ok(())
        })
    }
    
    // 开始扫描
    let future = scan_directory(root_dir, root_dir, &mut state, config, 0);
    future.await?;
    
    info!(
        "本地状态收集完成，共 {} 个条目 ({} 文件, {} 目录)",
        state.entry_count(),
        state.file_count(),
        state.directory_count()
    );
    
    Ok(state)
}

/// 保存文件系统状态到磁盘
pub async fn save_state(state: &FileSystemState, path: &Path) -> Result<()> {
    let json = serde_json::to_string_pretty(state)?;
    fs::write(path, json).await?;
    debug!("状态已保存到: {}", path.display());
    Ok(())
}

/// 从磁盘加载文件系统状态
pub async fn load_state(path: &Path) -> Result<FileSystemState> {
    let json = fs::read_to_string(path).await?;
    let state: FileSystemState = serde_json::from_str(&json)?;
    debug!("从 {} 加载了状态，共 {} 个条目", path.display(), state.entry_count());
    Ok(state)
}