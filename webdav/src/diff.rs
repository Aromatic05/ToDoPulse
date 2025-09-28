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

#[cfg(test)]
mod tests {
    use crate::diff::{compare_states, DiffConfig};
    use crate::model::{DiffType, EntryState, FileSystemState};

    use anyhow::Result;
    use chrono::{Duration, Utc};
    use std::path::PathBuf;

    // Helper function to create a state from a list of EntryState
    fn create_state(entries: Vec<EntryState>) -> FileSystemState {
        let mut state = FileSystemState::new();
        for entry in entries {
            state.add_entry(entry);
        }
        state
    }

    // --- 简单的测试用例 ---

    #[test]
    fn test_simple_empty_states() -> Result<()> {
        // 场景：本地和远程状态都为空，无差异
        let local = FileSystemState::new();
        let remote = FileSystemState::new();
        let config = DiffConfig::default();

        let diff = compare_states(&local, &remote, &config)?;

        assert_eq!(diff.added_count(), 0);
        assert_eq!(diff.deleted_count(), 0);
        assert_eq!(diff.modified_count(), 0);
        assert_eq!(diff.unchanged_count(), 0);
        assert_eq!(diff.entries.len(), 0);

        Ok(())
    }

    #[test]
    fn test_simple_unchanged_file() -> Result<()> {
        // 场景：一个文件在本地和远程都存在且相同 (大小和时间在容差内)
        let now = Utc::now();
        let entries = vec![EntryState::new_file("file1.txt".into(), now, 100)];

        let local = create_state(entries.clone());
        let remote = create_state(entries);
        let config = DiffConfig::default(); // time_tolerance: 1

        let diff = compare_states(&local, &remote, &config)?;

        assert_eq!(diff.added_count(), 0);
        assert_eq!(diff.deleted_count(), 0);
        assert_eq!(diff.modified_count(), 0);
        assert_eq!(diff.unchanged_count(), 1);
        assert_eq!(diff.entries.len(), 1);
        assert_eq!(diff.entries[0].diff_type, DiffType::Unchanged);

        Ok(())
    }

    #[test]
    fn test_simple_unchanged_dir() -> Result<()> {
        // 场景：一个目录在本地和远程都存在且相同
        let entries = vec![
        EntryState::new_directory("dir1".into(), Utc::now()), // 目录通常没有时间或大小
    ];

        let local = create_state(entries.clone());
        let remote = create_state(entries);
        let config = DiffConfig::default();

        let diff = compare_states(&local, &remote, &config)?;

        assert_eq!(diff.added_count(), 0);
        assert_eq!(diff.deleted_count(), 0);
        assert_eq!(diff.modified_count(), 0);
        assert_eq!(diff.unchanged_count(), 1);
        assert_eq!(diff.entries.len(), 1);
        assert_eq!(diff.entries[0].diff_type, DiffType::Unchanged);

        Ok(())
    }

    #[test]
    fn test_simple_added_file() -> Result<()> {
        // 场景：一个文件只在本地存在 (Added)
        let now = Utc::now();
        let local_entries = vec![EntryState::new_file("file_added.txt".into(), now, 150)];
        let remote_entries: Vec<EntryState> = vec![];

        let local = create_state(local_entries);
        let remote = create_state(remote_entries);
        let config = DiffConfig::default();

        let diff = compare_states(&local, &remote, &config)?;

        assert_eq!(diff.added_count(), 1);
        assert_eq!(diff.deleted_count(), 0);
        assert_eq!(diff.modified_count(), 0);
        assert_eq!(diff.unchanged_count(), 0);
        assert_eq!(diff.entries.len(), 1);
        assert_eq!(diff.entries[0].diff_type, DiffType::Added);
        assert_eq!(diff.entries[0].path, PathBuf::from("file_added.txt"));

        Ok(())
    }

    #[test]
    fn test_simple_deleted_file() -> Result<()> {
        // 场景：一个文件只在远程存在 (Deleted)
        let now = Utc::now();
        let local_entries: Vec<EntryState> = vec![];
        let remote_entries = vec![EntryState::new_file(
            "file_deleted.txt".into(),
            now,
            200,
        )];

        let local = create_state(local_entries);
        let remote = create_state(remote_entries);
        let config = DiffConfig::default();

        let diff = compare_states(&local, &remote, &config)?;

        assert_eq!(diff.added_count(), 0);
        assert_eq!(diff.deleted_count(), 1);
        assert_eq!(diff.modified_count(), 0);
        assert_eq!(diff.unchanged_count(), 0);
        assert_eq!(diff.entries.len(), 1);
        assert_eq!(diff.entries[0].diff_type, DiffType::Deleted);
        assert_eq!(diff.entries[0].path, PathBuf::from("file_deleted.txt"));

        Ok(())
    }

    #[test]
    fn test_simple_modified_size() -> Result<()> {
        // 场景：文件大小不同 (Modified)
        let now = Utc::now();
        let local_entries = vec![EntryState::new_file("file_mod.txt".into(), now, 100)];
        let remote_entries = vec![
        EntryState::new_file("file_mod.txt".into(), now, 101), // 大小不同
    ];

        let local = create_state(local_entries);
        let remote = create_state(remote_entries);
        let config = DiffConfig::default();

        let diff = compare_states(&local, &remote, &config)?;

        assert_eq!(diff.added_count(), 0);
        assert_eq!(diff.deleted_count(), 0);
        assert_eq!(diff.modified_count(), 1);
        assert_eq!(diff.unchanged_count(), 0);
        assert_eq!(diff.entries.len(), 1);
        assert_eq!(diff.entries[0].diff_type, DiffType::Modified);
        assert_eq!(diff.entries[0].path, PathBuf::from("file_mod.txt"));

        Ok(())
    }

    #[test]
    fn test_simple_modified_time() -> Result<()> {
        // 场景：文件时间不同 (Modified), 超出默认容差
        let now = Utc::now();
        let local_entries = vec![EntryState::new_file(
            "file_mod_time.txt".into(),
            now,
            100,
        )];
        let remote_entries = vec![EntryState::new_file(
            "file_mod_time.txt".into(),
            now + Duration::seconds(5),
            100,
        )];

        let local = create_state(local_entries);
        let remote = create_state(remote_entries);
        let config = DiffConfig::default(); // time_tolerance: 1

        let diff = compare_states(&local, &remote, &config)?;

        assert_eq!(diff.added_count(), 0);
        assert_eq!(diff.deleted_count(), 0);
        assert_eq!(diff.modified_count(), 1);
        assert_eq!(diff.unchanged_count(), 0);
        assert_eq!(diff.entries.len(), 1);
        assert_eq!(diff.entries[0].diff_type, DiffType::Modified);
        assert_eq!(
            diff.entries[0].path,
            PathBuf::from("file_mod_time.txt")
        );

        Ok(())
    }

    #[test]
    fn test_simple_modified_type() -> Result<()> {
        // 场景：同路径下，本地是文件，远程是目录 (Modified)
        let now = Utc::now();
        let local_entries = vec![EntryState::new_file("entry_mod_type".into(), now, 100)];
        let remote_entries = vec![EntryState::new_directory("entry_mod_type".into(), now)];

        let local = create_state(local_entries);
        let remote = create_state(remote_entries);
        let config = DiffConfig::default();

        let diff = compare_states(&local, &remote, &config)?;

        assert_eq!(diff.added_count(), 0);
        assert_eq!(diff.deleted_count(), 0);
        assert_eq!(diff.modified_count(), 1);
        assert_eq!(diff.unchanged_count(), 0);
        assert_eq!(diff.entries.len(), 1);
        assert_eq!(diff.entries[0].diff_type, DiffType::Modified);
        assert_eq!(diff.entries[0].path, PathBuf::from("entry_mod_type"));

        Ok(())
    }
}