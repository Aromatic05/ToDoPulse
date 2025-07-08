use crate::model::{EntryState, FileSystemState};
use anyhow::{Result, anyhow};
use log::{debug, error, info};
use path_clean::PathClean;
use reqwest_dav::{Auth, Client, ClientBuilder, Depth, list_cmd::ListEntity};
use std::path::{Path, PathBuf};
use tokio::fs;

/// 创建WebDAV客户端
pub async fn create_client(credential: (&str, &str, &str)) -> Result<Client> {
    debug!("创建WebDAV客户端: {}", credential.0);
    let (host, username, password) = credential;

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

    let client = create_client((host, username, password)).await?;

    // 尝试列出根目录内容来测试连接
    match client.list("/", Depth::Number(1)).await {
        Ok(_) => {
            debug!("WebDAV连接测试成功");
            Ok(true)
        }
        Err(e) => {
            error!("WebDAV连接测试失败: {}", e);
            Err(anyhow!("无法连接到WebDAV服务器: {}", e))
        }
    }
}

/// 确保远程目录存在
pub async fn ensure_remote_dir_exists(client: &Client, dir: &Path) -> Result<()> {
    debug!("确保远程目录存在: {}", dir.display());

    let dir = dir.clean();

    // 分割路径并逐级创建
    let mut current_path = PathBuf::new();
    for part in dir.components().filter(|c| c.as_os_str() != "/") {
        current_path.push(part);

        let path_str = &current_path.as_os_str().to_string_lossy();
        if client.list(path_str, Depth::Number(0)).await.is_err() {
            debug!("创建远程目录: {}", path_str);
            client.mkcol(path_str).await?;
        } else {
            debug!("远程目录已存在: {}", path_str);
        }
    }

    Ok(())
}

/// 上传文件到WebDAV服务器
pub async fn upload_file(client: &Client, local_path: &Path, remote_path: &Path) -> Result<()> {
    debug!("上传文件: {} -> {}", local_path.display(), remote_path.display());

    let remote_path = &remote_path.as_os_str().to_string_lossy();
    // 读取本地文件
    let content = fs::read(local_path).await?;

    // 上传到远程
    client.put(remote_path, content).await?;

    debug!("文件上传成功");
    Ok(())
}

/// 从WebDAV服务器下载文件
pub async fn download_file(client: &Client, remote_path: &Path, local_path: &Path) -> Result<()> {
    debug!("下载文件: {} -> {}", remote_path.display(), local_path.display());

    let remote_path = &remote_path.as_os_str().to_string_lossy();
    // 创建父目录（如果不存在）
    if let Some(parent) = local_path.parent() {
        fs::create_dir_all(parent).await?;
    }

    // 获取远程文件
    let response = client.get(remote_path).await?;
    let content = response.bytes().await?;

    // 写入本地文件
    fs::write(local_path, content).await?;

    debug!("文件下载成功");
    Ok(())
}

/// 收集远程文件系统状态
pub async fn collect_remote_state(client: &Client, remote_dir: &Path) -> Result<FileSystemState> {
    info!("收集远程文件系统状态: {}", remote_dir.display());

    let remote_dir = remote_dir.clean();

    ensure_remote_dir_exists(client, &remote_dir).await?;

    // 创建文件系统状态
    let mut state = FileSystemState::new();

    let remote_dir_str = remote_dir
        .to_str()
        .ok_or_else(|| anyhow!("无法将远程目录转换为字符串: {}", remote_dir.display()))?;

    // 递归列出所有文件和目录
    let entities = client
        .list(remote_dir_str, Depth::Infinity)
        .await
        .map_err(|e| anyhow!("列出远程目录失败: {}", e))?;

    // 处理每个条目
    for entity in entities {
        let href = match &entity {
            ListEntity::File(file) => &file.href,
            ListEntity::Folder(folder) => &folder.href,
        };

        if href == remote_dir_str {
            continue;
        }

        let rel_path = PathBuf::from(format!(
            "/{}",
            href.strip_prefix(remote_dir_str)
                .unwrap_or(&href)
                .trim_start_matches('/')
        ));

        let entry_state = match &entity {
            ListEntity::File(file) => {
                EntryState::new_file(rel_path, file.last_modified, file.content_length as u64)
            }
            ListEntity::Folder(folder) => EntryState::new_directory(rel_path, folder.last_modified),
        };

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
