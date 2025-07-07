use crate::model::{DiffEntry, DiffResult, DiffType, EntryState, FileSystemState};
use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use log::info;

/// 差异比较配置
pub struct DiffConfig {
    /// 修改时间容差（秒）
    pub time_tolerance: i64,

    /// 是否比较文件内容哈希
    pub compare_hash: bool,

    /// 是否忽略修改时间，仅比较大小和/或哈希
    pub ignore_time: bool,
}

impl Default for DiffConfig {
    fn default() -> Self {
        Self {
            time_tolerance: 1,
            compare_hash: false,
            ignore_time: false,
        }
    }
}

/// 比较两个时间戳是否相等（考虑容差）
fn time_equals(t1: &DateTime<Utc>, t2: &DateTime<Utc>, tolerance: i64) -> bool {
    let duration = t1.signed_duration_since(*t2).abs();
    duration <= Duration::seconds(tolerance)
}

/// 比较两个文件系统状态，生成差异报告
pub fn compare_states(
    local: &FileSystemState,
    remote: &FileSystemState,
    config: &DiffConfig,
) -> Result<DiffResult> {
    info!(
        "比较文件系统状态: 本地 {} 个条目, 远程 {} 个条目",
        local.entry_count(),
        remote.entry_count()
    );

    let mut result = DiffResult::new();
    let mut processed_paths = std::collections::HashSet::new();

    // 第一步：遍历本地状态，与远程对比
    for (path, local_entry) in &local.entries {
        processed_paths.insert(path);

        let remote_entry = remote.entries.get(path);
        let diff_type = compare_entries(local_entry, remote_entry, config);
        let remote_state = remote_entry.cloned();
        result.add_entry(DiffEntry {
            path: path.clone(),
            diff_type,
            entry_type: local_entry.entry_type.clone(),
            local_state: Some(local_entry.clone()),
            remote_state,
        });
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

// compare_states 的辅助函数
fn compare_entries(
    local_entry: &EntryState,
    remote_entry: Option<&EntryState>,
    config: &DiffConfig,
) -> DiffType {
    let Some(remote_entry) = remote_entry else {
        return DiffType::Added;
    };
    match (local_entry.is_directory(), remote_entry.is_directory()) {
        (true, true) => DiffType::Unchanged,
        (false, false) => {
            if check_same(local_entry, remote_entry, config) {
                DiffType::Unchanged
            } else {
                DiffType::Modified
            }
        }
        _ => DiffType::Modified,
    }
}

// 检查一致性
fn check_same(
    local: &EntryState,
    remote: &EntryState,
    config: &DiffConfig,
) -> bool {
    let size_same = local.size == remote.size;
    let time_same = config.ignore_time || time_equals(
        &local.modified,
        &remote.modified,
        config.time_tolerance,
    );

    size_same && (time_same || (config.compare_hash && 
        matches!((&local.content_hash, &remote.content_hash), 
            (Some(h1), Some(h2)) if h1 == h2)))
}