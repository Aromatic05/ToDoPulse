use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// 文件/目录类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntryType {
    File,
    Directory,
}

/// 表示一个文件或目录的状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryState {
    /// 相对路径（以/开头）
    pub path: String,
    /// 条目类型（文件或目录）
    pub entry_type: EntryType,
    /// 修改时间
    pub modified: Option<DateTime<Utc>>,
    /// 文件大小（仅对文件有效）
    pub size: Option<u64>,
    /// 文件内容哈希值（可选，仅对文件有效）
    pub content_hash: Option<String>,
}

impl EntryState {
    /// 创建新的文件状态
    pub fn new_file(path: String, modified: Option<DateTime<Utc>>, size: Option<u64>) -> Self {
        Self {
            path,
            entry_type: EntryType::File,
            modified,
            size,
            content_hash: None,
        }
    }

    /// 创建新的目录状态
    pub fn new_directory(path: String, modified: Option<DateTime<Utc>>) -> Self {
        Self {
            path,
            entry_type: EntryType::Directory,
            modified,
            size: None,
            content_hash: None,
        }
    }

    /// 设置内容哈希值
    pub fn with_hash(mut self, hash: String) -> Self {
        self.content_hash = Some(hash);
        self
    }

    /// 检查是否为文件
    pub fn is_file(&self) -> bool {
        matches!(self.entry_type, EntryType::File)
    }

    /// 检查是否为目录
    pub fn is_directory(&self) -> bool {
        matches!(self.entry_type, EntryType::Directory)
    }
}

/// 表示文件系统状态的集合
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSystemState {
    /// 条目集合，键为相对路径
    pub entries: HashMap<String, EntryState>,
    /// 状态收集时间
    pub collection_time: DateTime<Utc>,
}

impl FileSystemState {
    /// 创建新的文件系统状态
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            collection_time: Utc::now(),
        }
    }

    /// 添加条目
    pub fn add_entry(&mut self, entry: EntryState) {
        self.entries.insert(entry.path.clone(), entry);
    }

    /// 获取条目数量
    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

    /// 获取文件数量
    pub fn file_count(&self) -> usize {
        self.entries.values().filter(|e| e.is_file()).count()
    }

    /// 获取目录数量
    pub fn directory_count(&self) -> usize {
        self.entries.values().filter(|e| e.is_directory()).count()
    }
}

/// 表示同步差异的类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiffType {
    /// 添加了新条目
    Added,
    /// 删除了条目
    Deleted,
    /// 修改了条目
    Modified,
    /// 条目保持不变
    Unchanged,
}

/// 表示两个文件系统状态之间的差异
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffEntry {
    /// 条目路径
    pub path: String,
    /// 差异类型
    pub diff_type: DiffType,
    /// 条目类型
    pub entry_type: EntryType,
    /// 本地状态
    pub local_state: Option<EntryState>,
    /// 远程状态
    pub remote_state: Option<EntryState>,
}

impl DiffEntry {
    /// 检查是否为文件
    pub fn is_file(&self) -> bool {
        self.entry_type == EntryType::File
    }

    /// 检查是否为目录
    pub fn is_directory(&self) -> bool {
        self.entry_type == EntryType::Directory
    }
}

/// 所有差异的集合
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffResult {
    /// 差异条目列表
    pub entries: Vec<DiffEntry>,
    /// 比较时间
    pub diff_time: DateTime<Utc>,
}

impl DiffResult {
    /// 创建新的差异结果
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            diff_time: Utc::now(),
        }
    }

    /// 添加差异条目
    pub fn add_entry(&mut self, entry: DiffEntry) {
        self.entries.push(entry);
    }

    /// 获取添加的条目数量
    pub fn added_count(&self) -> usize {
        self.entries
            .iter()
            .filter(|e| e.diff_type == DiffType::Added)
            .count()
    }

    /// 获取删除的条目数量
    pub fn deleted_count(&self) -> usize {
        self.entries
            .iter()
            .filter(|e| e.diff_type == DiffType::Deleted)
            .count()
    }

    /// 获取修改的条目数量
    pub fn modified_count(&self) -> usize {
        self.entries
            .iter()
            .filter(|e| e.diff_type == DiffType::Modified)
            .count()
    }

    /// 获取未变更的条目数量
    pub fn unchanged_count(&self) -> usize {
        self.entries
            .iter()
            .filter(|e| e.diff_type == DiffType::Unchanged)
            .count()
    }
}

/// 同步冲突解决策略
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConflictStrategy {
    /// 优先使用本地版本
    PreferLocal,
    /// 优先使用远程版本
    PreferRemote,
    /// 保留两者（重命名本地版本）
    KeepBoth,
    /// 跳过冲突文件
    Skip,
}

/// 同步操作的类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SyncOperationType {
    /// 上传文件
    Upload,
    /// 下载文件
    Download,
    /// 删除本地文件
    DeleteLocal,
    /// 删除远程文件
    DeleteRemote,
    /// 创建本地目录
    CreateLocalDirectory,
    /// 创建远程目录
    CreateRemoteDirectory,
    /// 跳过（不操作）
    Skip,
}

/// 表示一个同步操作
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncOperation {
    /// 操作类型
    pub operation_type: SyncOperationType,
    /// 相对路径
    pub path: String,
    /// 条目类型
    pub entry_type: EntryType,
    /// 操作状态
    pub status: SyncOperationStatus,
    /// 错误信息（如果有）
    pub error: Option<String>,
}

/// 同步操作的状态
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SyncOperationStatus {
    /// 待执行
    Pending,
    /// 正在执行
    InProgress,
    /// 已完成
    Completed,
    /// 失败
    Failed,
    /// 跳过
    Skipped,
}

/// 同步会话的状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncSession {
    /// 会话ID
    pub id: String,
    /// 会话开始时间
    pub start_time: DateTime<Utc>,
    /// 会话结束时间
    pub end_time: Option<DateTime<Utc>>,
    /// 同步操作列表
    pub operations: Vec<SyncOperation>,
    /// 本地目录
    pub local_dir: PathBuf,
    /// 远程目录
    pub remote_dir: String,
    /// 会话状态
    pub status: SyncSessionStatus,
    /// 错误信息（如果有）
    pub error: Option<String>,
}

/// 同步会话的状态
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SyncSessionStatus {
    /// 正在准备中
    Preparing,
    /// 正在收集状态
    CollectingState,
    /// 正在比较差异
    Diffing,
    /// 正在计划操作
    Planning,
    /// 正在执行操作
    Executing,
    /// 已完成
    Completed,
    /// 已失败
    Failed,
    /// 已中止
    Aborted,
}

impl SyncSession {
    /// 创建新的同步会话
    pub fn new(local_dir: PathBuf, remote_dir: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            start_time: Utc::now(),
            end_time: None,
            operations: Vec::new(),
            local_dir,
            remote_dir,
            status: SyncSessionStatus::Preparing,
            error: None,
        }
    }

    /// 添加操作
    pub fn add_operation(&mut self, operation: SyncOperation) {
        self.operations.push(operation);
    }

    /// 完成会话
    pub fn complete(&mut self) {
        self.status = SyncSessionStatus::Completed;
        self.end_time = Some(Utc::now());
    }

    /// 会话失败
    #[allow(dead_code)]
    pub fn fail(&mut self, error: String) {
        self.status = SyncSessionStatus::Failed;
        self.error = Some(error);
        self.end_time = Some(Utc::now());
    }

    /// 中止会话
    #[allow(dead_code)]
    pub fn abort(&mut self, reason: String) {
        self.status = SyncSessionStatus::Aborted;
        self.error = Some(reason);
        self.end_time = Some(Utc::now());
    }

    /// 获取会话的统计信息
    pub fn get_stats(&self) -> SyncSessionStats {
        let completed = self
            .operations
            .iter()
            .filter(|op| op.status == SyncOperationStatus::Completed)
            .count();

        let failed = self
            .operations
            .iter()
            .filter(|op| op.status == SyncOperationStatus::Failed)
            .count();

        let skipped = self
            .operations
            .iter()
            .filter(|op| op.status == SyncOperationStatus::Skipped)
            .count();

        let uploaded = self
            .operations
            .iter()
            .filter(|op| {
                op.status == SyncOperationStatus::Completed
                    && op.operation_type == SyncOperationType::Upload
            })
            .count();

        let downloaded = self
            .operations
            .iter()
            .filter(|op| {
                op.status == SyncOperationStatus::Completed
                    && op.operation_type == SyncOperationType::Download
            })
            .count();

        let duration = if let Some(end) = self.end_time {
            end.signed_duration_since(self.start_time).num_seconds()
        } else {
            Utc::now()
                .signed_duration_since(self.start_time)
                .num_seconds()
        };

        SyncSessionStats {
            total: self.operations.len(),
            completed,
            failed,
            skipped,
            uploaded,
            downloaded,
            duration_seconds: duration,
        }
    }
}

/// 同步会话的统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncSessionStats {
    /// 总操作数
    pub total: usize,
    /// 已完成操作数
    pub completed: usize,
    /// 失败操作数
    pub failed: usize,
    /// 跳过操作数
    pub skipped: usize,
    /// 上传文件数
    pub uploaded: usize,
    /// 下载文件数
    pub downloaded: usize,
    /// 持续时间（秒）
    pub duration_seconds: i64,
}
