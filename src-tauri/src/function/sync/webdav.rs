use anyhow::{Result, anyhow};
use log::{info, warn, error, debug};
use reqwest_dav::{list_cmd::{ListEntity, ListFolder, ListFile}, Auth, Client, ClientBuilder, Depth};
use std::path::{Path, PathBuf};
use tokio::fs;
use std::collections::HashMap;
use chrono::{DateTime, Utc};

// 定义我们自己的文件系统条目类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntryType {
    File,
    Directory,
}

// 定义我们自己的文件系统条目
#[derive(Debug, Clone)]
pub struct EntryState {
    pub path: String,
    pub entry_type: EntryType,
    pub modified: Option<DateTime<Utc>>,
    pub size: Option<u64>,
}

impl EntryState {
    pub fn new_file(path: String, modified: Option<DateTime<Utc>>, size: Option<u64>) -> Self {
        Self {
            path,
            entry_type: EntryType::File,
            modified,
            size,
        }
    }

    pub fn new_directory(path: String, modified: Option<DateTime<Utc>>) -> Self {
        Self {
            path,
            entry_type: EntryType::Directory,
            modified,
            size: None,
        }
    }

    pub fn is_file(&self) -> bool {
        self.entry_type == EntryType::File
    }

    pub fn is_directory(&self) -> bool {
        self.entry_type == EntryType::Directory
    }
}

// 定义文件系统状态
#[derive(Debug, Clone)]
pub struct FileSystemState {
    pub entries: HashMap<String, EntryState>,
    pub collection_time: DateTime<Utc>,
}

impl FileSystemState {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            collection_time: Utc::now(),
        }
    }

    pub fn add_entry(&mut self, entry: EntryState) {
        self.entries.insert(entry.path.clone(), entry);
    }

    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

    pub fn file_count(&self) -> usize {
        self.entries.values().filter(|e| e.is_file()).count()
    }

    pub fn directory_count(&self) -> usize {
        self.entries.values().filter(|e| e.is_directory()).count()
    }
}

/// 创建WebDAV客户端
pub async fn create_client(host: &str, username: &str, password: &str) -> Result<Client> {
    debug!("创建WebDAV客户端: {}", host);
    
    // 创建WebDAV客户端
    let client = ClientBuilder::new()
        .set_host(host.to_string())
        .set_auth(Auth::Basic(username.to_owned(), password.to_owned()))
        .build()?;
    
    Ok(client)
}

/// 测试WebDAV连接
pub async fn test_connection(host: &str, username: &str, password: &str) -> Result<bool> {
    debug!("测试WebDAV连接: {}", host);
    
    let client = create_client(host, username, password).await?;
    
    // 尝试列出根目录内容来测试连接
    match client.list("/", Depth::Number(1)).await {
        Ok(_) => {
            debug!("WebDAV连接测试成功");
            Ok(true)
        },
        Err(e) => {
            error!("WebDAV连接测试失败: {}", e);
            Err(anyhow!("无法连接到WebDAV服务器: {}", e))
        }
    }
}

/// 确保远程目录存在
pub async fn ensure_remote_dir_exists(client: &Client, dir: &str) -> Result<()> {
    debug!("确保远程目录存在: {}", dir);
    
    // 规范化路径
    let dir = normalize_path(dir);
    
    // 根目录总是存在的
    if dir == "/" {
        return Ok(());
    }
    
    // 分割路径并逐级创建
    let parts: Vec<&str> = dir.split('/').filter(|s| !s.is_empty()).collect();
    let mut current_path = String::new();
    
    for part in parts {
        current_path.push('/');
        current_path.push_str(part);
        
        // 尝试列出目录内容来检查是否存在
        match client.list(&current_path, Depth::Number(0)).await {
            Ok(_) => {
                debug!("远程目录已存在: {}", current_path);
            },
            Err(_) => {
                debug!("创建远程目录: {}", current_path);
                client.mkcol(&current_path).await?;
            }
        }
    }
    
    Ok(())
}

/// 上传文件到WebDAV服务器
pub async fn upload_file(client: &Client, local_path: &Path, remote_path: &str) -> Result<()> {
    debug!("上传文件: {} -> {}", local_path.display(), remote_path);
    
    // 规范化路径
    let remote_path = normalize_path(remote_path);
    
    // 读取本地文件内容
    let content = fs::read(local_path).await?;
    
    // 上传到远程
    client.put(&remote_path, content).await?;
    
    debug!("文件上传成功");
    Ok(())
}

/// 从WebDAV服务器下载文件
pub async fn download_file(client: &Client, remote_path: &str, local_path: &Path) -> Result<()> {
    debug!("下载文件: {} -> {}", remote_path, local_path.display());
    
    // 规范化路径
    let remote_path = normalize_path(remote_path);
    
    // 创建父目录（如果不存在）
    if let Some(parent) = local_path.parent() {
        fs::create_dir_all(parent).await?;
    }
    
    // 获取远程文件
    let response = client.get(&remote_path).await?;
    let content = response.bytes().await?;
    
    // 写入本地文件
    fs::write(local_path, content).await?;
    
    debug!("文件下载成功");
    Ok(())
}

/// 收集远程文件系统状态
pub async fn collect_remote_state(
    client: &Client, 
    remote_dir: &str
) -> Result<FileSystemState> {
    info!("收集远程文件系统状态: {}", remote_dir);
    
    // 规范化路径
    let remote_dir = normalize_path(remote_dir);
    
    // 确保远程目录存在
    ensure_remote_dir_exists(client, &remote_dir).await?;
    
    // 创建文件系统状态
    let mut state = FileSystemState::new();
    
    // 递归列出所有文件和目录
    let entities = match client.list(&remote_dir, Depth::Infinity).await {
        Ok(entities) => entities,
        Err(e) => {
            error!("列出远程目录内容失败: {} - {}", remote_dir, e);
            return Err(anyhow!("无法获取远程文件列表: {}", e));
        }
    };
    
    // 处理每个条目
    for entity in entities {
        // 从枚举中获取href和其他信息
        let (href, is_folder, modified, size) = match &entity {
            ListEntity::File(file) => {
                let modified = Some(file.last_modified);
                let size = Some(file.content_length as u64);
                (file.href.clone(), false, modified, size)
            },
            ListEntity::Folder(folder) => {
                let modified = Some(folder.last_modified);
                (folder.href.clone(), true, modified, None)
            }
        };
        
        // 跳过当前目录
        if href == remote_dir {
            continue;
        }
        
        // 计算相对路径
        let rel_path = if href.starts_with(&remote_dir) {
            if remote_dir == "/" {
                href.clone()
            } else {
                let suffix = &href[remote_dir.len()..];
                if suffix.is_empty() {
                    "/".to_string()
                } else {
                    suffix.to_string()
                }
            }
        } else {
            href.clone()
        };
        
        // 确保路径以/开头
        let rel_path = if !rel_path.starts_with('/') {
            format!("/{}", rel_path)
        } else {
            rel_path
        };
        
        // 创建条目状态
        let entry_state = if is_folder {
            EntryState::new_directory(rel_path, modified)
        } else {
            EntryState::new_file(rel_path, modified, size)
        };
        
        // 添加到状态
        state.add_entry(entry_state);
    }
    
    info!(
        "远程状态收集完成，共 {} 个条目 ({} 文件, {} 目录)",
        state.entry_count(),
        state.file_count(),
        state.directory_count()
    );
    
    Ok(state)
}

/// 同步目录
pub async fn sync_directory(
    host: &str,
    username: &str,
    password: &str,
    local_dir: &PathBuf,
    remote_dir: &str
) -> Result<()> {
    info!("同步目录: {} <-> {}", local_dir.display(), remote_dir);
    
    // 创建WebDAV客户端
    let client = create_client(host, username, password).await?;
    
    // 规范化远程目录路径
    let remote_dir = normalize_path(remote_dir);
    
    // 确保远程目录存在
    ensure_remote_dir_exists(&client, &remote_dir).await?;
    
    // 获取远程文件列表
    let remote_state = collect_remote_state(&client, &remote_dir).await?;
    
    // 获取本地文件列表
    let local_state = collect_local_state(local_dir).await?;
    
    // 比较差异并执行同步
    sync_states(&client, local_dir, &remote_dir, &local_state, &remote_state).await?;
    
    info!("同步完成");
    Ok(())
}

/// 收集本地文件系统状态
async fn collect_local_state(dir: &Path) -> Result<FileSystemState> {
    debug!("收集本地文件系统状态: {}", dir.display());
    
    // 检查目录是否存在
    if !dir.exists() {
        return Err(anyhow!("目录不存在: {}", dir.display()));
    }
    
    if !dir.is_dir() {
        return Err(anyhow!("路径不是目录: {}", dir.display()));
    }
    
    let mut state = FileSystemState::new();
    
    // 递归遍历目录收集状态
    collect_dir_entries(dir, dir, &mut state).await?;
    
    debug!(
        "本地状态收集完成，共 {} 个条目 ({} 文件, {} 目录)",
        state.entry_count(),
        state.file_count(),
        state.directory_count()
    );
    
    Ok(state)
}

/// 处理单个文件或目录
async fn process_fs_entry(
    root: &Path,
    path: &Path,
    state: &mut FileSystemState,
    dirs_to_process: &mut Vec<PathBuf>
) -> Result<()> {
    let metadata = fs::metadata(path).await?;
    
    // 计算相对路径
    let rel_path = path.strip_prefix(root)
        .map_err(|_| anyhow!("无法创建相对路径"))?;
        
    let rel_path_str = format!("/{}", rel_path.to_string_lossy().replace('\\', "/"));
    
    if metadata.is_dir() {
        // 添加目录
        let modified = metadata.modified().ok()
            .map(|t| {
                let dt: DateTime<Utc> = t.into();
                dt
            });
            
        state.add_entry(EntryState::new_directory(rel_path_str, modified));
        
        // 将目录添加到待处理列表
        dirs_to_process.push(path.to_path_buf());
    } else if metadata.is_file() {
        // 添加文件
        let modified = metadata.modified().ok()
            .map(|t| {
                let dt: DateTime<Utc> = t.into();
                dt
            });
            
        let size = Some(metadata.len());
        
        state.add_entry(EntryState::new_file(rel_path_str, modified, size));
    }
    
    Ok(())
}

/// 收集目录条目（非递归实现）
async fn collect_dir_entries(
    root: &Path,
    dir: &Path,
    state: &mut FileSystemState
) -> Result<()> {
    // 使用Vec作为目录栈，避免递归
    let mut dirs_to_process = vec![dir.to_path_buf()];
    
    while let Some(current_dir) = dirs_to_process.pop() {
        let mut entries = fs::read_dir(&current_dir).await?;
        
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            process_fs_entry(root, &path, state, &mut dirs_to_process).await?;
        }
    }
    
    Ok(())
}

/// 比较状态并执行同步
async fn sync_states(
    client: &Client,
    local_dir: &Path,
    remote_dir: &str,
    local_state: &FileSystemState,
    remote_state: &FileSystemState
) -> Result<()> {
    debug!("比较状态并执行同步");
    
    let mut upload_count = 0;
    let mut download_count = 0;
    let mut skip_count = 0;
    
    // 1. 处理本地文件上传到远程
    for (path, local_entry) in &local_state.entries {
        if let Some(remote_entry) = remote_state.entries.get(path) {
            // 文件存在于两边，检查是否需要更新
            if local_entry.is_file() && remote_entry.is_file() {
                // 比较修改时间，如果本地更新则上传
                let should_upload = match (&local_entry.modified, &remote_entry.modified) {
                    (Some(local_time), Some(remote_time)) => local_time > remote_time,
                    (Some(_), None) => true, // 本地有时间戳但远程没有，上传
                    (None, Some(_)) => false, // 本地没有时间戳但远程有，跳过
                    (None, None) => {
                        // 两边都没有时间戳，比较大小
                        match (local_entry.size, remote_entry.size) {
                            (Some(local_size), Some(remote_size)) => local_size != remote_size,
                            _ => false, // 无法比较，跳过
                        }
                    }
                };
                
                if should_upload {
                    info!("更新远程文件: {}", path);
                    let local_path = local_dir.join(path.trim_start_matches('/'));
                    let remote_path = format!("{}{}", remote_dir, path);
                    
                    if let Err(e) = upload_file(client, &local_path, &remote_path).await {
                        warn!("上传文件失败: {} - {}", path, e);
                    } else {
                        upload_count += 1;
                    }
                } else {
                    debug!("文件无需更新: {}", path);
                    skip_count += 1;
                }
            }
        } else {
            // 远程不存在此文件，上传
            if local_entry.is_file() {
                info!("上传新文件: {}", path);
                let local_path = local_dir.join(path.trim_start_matches('/'));
                let remote_path = format!("{}{}", remote_dir, path);
                
                if let Err(e) = upload_file(client, &local_path, &remote_path).await {
                    warn!("上传文件失败: {} - {}", path, e);
                } else {
                    upload_count += 1;
                }
            } else if local_entry.is_directory() {
                // 创建远程目录
                info!("创建远程目录: {}", path);
                let remote_path = format!("{}{}", remote_dir, path);
                
                if let Err(e) = ensure_remote_dir_exists(client, &remote_path).await {
                    warn!("创建远程目录失败: {} - {}", path, e);
                }
            }
        }
    }
    
    // 2. 处理远程文件下载到本地
    for (path, remote_entry) in &remote_state.entries {
        if !local_state.entries.contains_key(path) {
            // 本地不存在此文件，下载
            if remote_entry.is_file() {
                info!("下载新文件: {}", path);
                let local_path = local_dir.join(path.trim_start_matches('/'));
                let remote_path = format!("{}{}", remote_dir, path);
                
                if let Err(e) = download_file(client, &remote_path, &local_path).await {
                    warn!("下载文件失败: {} - {}", path, e);
                } else {
                    download_count += 1;
                }
            } else if remote_entry.is_directory() {
                // 创建本地目录
                info!("创建本地目录: {}", path);
                let local_path = local_dir.join(path.trim_start_matches('/'));
                
                if let Err(e) = fs::create_dir_all(&local_path).await {
                    warn!("创建本地目录失败: {} - {}", path, e);
                }
            }
        }
    }
    
    info!("同步完成: 上传 {} 个文件, 下载 {} 个文件, 跳过 {} 个文件", 
         upload_count, download_count, skip_count);
    
    Ok(())
}

/// 规范化路径
///
/// 处理以下情况:
/// 1. 将反斜杠替换为正斜杠
/// 2. 确保路径以斜杠开头
/// 3. 移除尾部斜杠（除了根目录）
/// 4. 处理连续的斜杠
/// 5. 解析 "." 和 ".." 路径元素
fn normalize_path(path: &str) -> String {
    let mut result = path.replace('\\', "/");
    
    // 确保路径以/开头
    if !result.starts_with('/') {
        result = format!("/{}", result);
    }
    
    // 移除尾部斜杠（除了根目录）
    if result.len() > 1 && result.ends_with('/') {
        result.pop();
    }
    
    // 处理连续的斜杠
    while result.contains("//") {
        result = result.replace("//", "/");
    }
    
    // 处理 "." 和 ".." 路径元素
    let mut components = Vec::new();
    for component in result.split('/').filter(|s| !s.is_empty()) {
        match component {
            "." => {}, // 忽略当前目录符号
            ".." => {
                // 回到上一级目录
                if !components.is_empty() {
                    components.pop();
                }
            },
            _ => components.push(component),
        }
    }
    
    // 重新构建路径
    let normalized = if components.is_empty() {
        "/".to_string()
    } else {
        format!("/{}", components.join("/"))
    };
    
    normalized
}
