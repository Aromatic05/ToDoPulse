use crate::function::sync::diff::{compare_states, DiffConfig};
use crate::function::sync::model::{
    FileSystemState, EntryState, DiffType
};

use chrono::{Utc, Duration};
use anyhow::Result;

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
    let entries = vec![
        EntryState::new_file("file1.txt".to_string(), Some(now), Some(100)),
    ];

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
        EntryState::new_directory("dir1".to_string(), None), // 目录通常没有时间或大小
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
    let local_entries = vec![
        EntryState::new_file("file_added.txt".to_string(), Some(now), Some(150)),
    ];
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
    assert_eq!(diff.entries[0].path, "file_added.txt".to_string());

    Ok(())
}

#[test]
fn test_simple_deleted_file() -> Result<()> {
    // 场景：一个文件只在远程存在 (Deleted)
    let now = Utc::now();
    let local_entries: Vec<EntryState> = vec![];
    let remote_entries = vec![
        EntryState::new_file("file_deleted.txt".to_string(), Some(now), Some(200)),
    ];

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
    assert_eq!(diff.entries[0].path, "file_deleted.txt".to_string());

    Ok(())
}

#[test]
fn test_simple_modified_size() -> Result<()> {
    // 场景：文件大小不同 (Modified)
    let now = Utc::now();
    let local_entries = vec![
        EntryState::new_file("file_mod.txt".to_string(), Some(now), Some(100)),
    ];
    let remote_entries = vec![
        EntryState::new_file("file_mod.txt".to_string(), Some(now), Some(101)), // 大小不同
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
    assert_eq!(diff.entries[0].path, "file_mod.txt".to_string());

    Ok(())
}

#[test]
fn test_simple_modified_time() -> Result<()> {
    // 场景：文件时间不同 (Modified), 超出默认容差
    let now = Utc::now();
    let local_entries = vec![
        EntryState::new_file("file_mod_time.txt".to_string(), Some(now), Some(100)),
    ];
    let remote_entries = vec![
        EntryState::new_file("file_mod_time.txt".to_string(), Some(now + Duration::seconds(5)), Some(100)), // 时间不同
    ];

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
    assert_eq!(diff.entries[0].path, "file_mod_time.txt".to_string());

    Ok(())
}

#[test]
fn test_simple_modified_type() -> Result<()> {
    // 场景：同路径下，本地是文件，远程是目录 (Modified)
    let now = Utc::now();
    let local_entries = vec![
        EntryState::new_file("entry_mod_type".to_string(), Some(now), Some(100)),
    ];
    let remote_entries = vec![
        EntryState::new_directory("entry_mod_type".to_string(), Some(now)),
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
    assert_eq!(diff.entries[0].path, "entry_mod_type".to_string());

    Ok(())
}