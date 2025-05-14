use anyhow::{Result, anyhow};
use log::{info, error, debug};
use reqwest_dav::{list_cmd::{ListEntity}, Auth, Client, ClientBuilder, Depth};
use std::path::{Path};
use tokio::fs;
use crate::function::sync::model::{FileSystemState, EntryState};

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

/// 规范化路径
///
/// 处理以下情况:
/// 1. 将反斜杠替换为正斜杠
/// 2. 确保路径以斜杠开头
/// 3. 移除尾部斜杠（除了根目录）
/// 4. 处理连续的斜杠
/// 5. 解析 "." 和 ".." 路径元素
pub(crate) fn normalize_path(path: &str) -> String {
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
