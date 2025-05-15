use anyhow::Result;
use log::{info, debug};
use chrono::{DateTime, Utc, Duration};
use crate::function::sync::model::{
    FileSystemState, DiffResult, DiffEntry, DiffType
};

/// 差异比较配置
pub struct DiffConfig {
    /// 修改时间容差（秒）
    /// 有些文件系统的时间戳精度可能不同，允许一定的误差
    pub time_tolerance: i64,
    
    /// 是否比较文件内容哈希
    /// 如果为true，则需要两个状态中都包含内容哈希
    pub compare_hash: bool,
    
    /// 是否忽略修改时间，仅比较大小和/或哈希
    pub ignore_time: bool,
}

impl Default for DiffConfig {
    fn default() -> Self {
        Self {
            time_tolerance: 1, // 1秒容差
            compare_hash: false,
            ignore_time: false,
        }
    }
}

/// 比较两个时间戳是否相等（考虑容差）
fn time_equals(t1: &Option<DateTime<Utc>>, t2: &Option<DateTime<Utc>>, tolerance: i64) -> bool {
    match (t1, t2) {
        (Some(time1), Some(time2)) => {
            let duration = if time1 > time2 {
                time1.signed_duration_since(*time2)
            } else {
                time2.signed_duration_since(*time1)
            };
            
            duration <= Duration::seconds(tolerance)
        },
        // 如果有一个时间戳缺失，则认为不相等
        _ => false,
    }
}

/// 比较两个文件系统状态，生成差异报告
pub fn compare_states(
    local: &FileSystemState, 
    remote: &FileSystemState,
    config: &DiffConfig
) -> Result<DiffResult> {
    debug!("比较文件系统状态: 本地 {} 个条目, 远程 {} 个条目", 
           local.entry_count(), remote.entry_count());
    
    let mut result = DiffResult::new();
    let mut processed_paths = std::collections::HashSet::new();
    
    // 第一步：遍历本地状态，与远程对比
    for (path, local_entry) in &local.entries {
        processed_paths.insert(path.clone());
        
        if let Some(remote_entry) = remote.entries.get(path) {
            // 条目在两端都存在，检查是否有差异
            let is_same = if local_entry.is_directory() && remote_entry.is_directory() {
                // 对于目录，仅检查类型是否相同
                true
            } else if local_entry.is_file() && remote_entry.is_file() {
                // 对于文件，检查大小和修改时间
                let size_same = local_entry.size == remote_entry.size;
                
                let time_same = if config.ignore_time {
                    true
                } else {
                    time_equals(
                        &local_entry.modified, 
                        &remote_entry.modified, 
                        config.time_tolerance
                    )
                };
                
                // 如果配置为比较哈希值
                if config.compare_hash {
                    let hash_same = match (&local_entry.content_hash, &remote_entry.content_hash) {
                        (Some(hash1), Some(hash2)) => hash1 == hash2,
                        _ => false,
                    };
                    size_same && (time_same || hash_same)
                } else {
                    // 不比较哈希值时，只考虑大小和时间
                    size_same && time_same
                }
            } else {
                // 类型不同（一个是文件，一个是目录）
                false
            };
            
            if is_same {
                // 条目相同
                result.add_entry(DiffEntry {
                    path: path.clone(),
                    diff_type: DiffType::Unchanged,
                    entry_type: local_entry.entry_type.clone(),
                    local_state: Some(local_entry.clone()),
                    remote_state: Some(remote_entry.clone()),
                });
            } else {
                // 条目被修改
                result.add_entry(DiffEntry {
                    path: path.clone(),
                    diff_type: DiffType::Modified,
                    entry_type: local_entry.entry_type.clone(),
                    local_state: Some(local_entry.clone()),
                    remote_state: Some(remote_entry.clone()),
                });
            }
        } else {
            // 条目只在本地存在，远程不存在
            result.add_entry(DiffEntry {
                path: path.clone(),
                diff_type: DiffType::Added,
                entry_type: local_entry.entry_type.clone(),
                local_state: Some(local_entry.clone()),
                remote_state: None,
            });
        }
    }
    
    // 第二步：查找仅存在于远程的条目
    for (path, remote_entry) in &remote.entries {
        if !processed_paths.contains(path) {
            // 条目只在远程存在，本地不存在
            result.add_entry(DiffEntry {
                path: path.clone(),
                diff_type: DiffType::Deleted,
                entry_type: remote_entry.entry_type.clone(),
                local_state: None,
                remote_state: Some(remote_entry.clone()),
            });
        }
    }
    
    // 记录差异统计信息
    info!(
        "差异比较完成: 添加 {}, 删除 {}, 修改 {}, 未变更 {}",
        result.added_count(),
        result.deleted_count(),
        result.modified_count(),
        result.unchanged_count()
    );
    
    Ok(result)
}
