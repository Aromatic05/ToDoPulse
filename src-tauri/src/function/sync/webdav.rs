use crate::function::sync::model::{EntryState, FileSystemState};
use anyhow::{anyhow, Result};
use log::{debug, error, info};
use reqwest_dav::{list_cmd::ListEntity, Auth, Client, ClientBuilder, Depth};
use std::path::Path;
use tokio::fs;

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
        }
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
            }
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
pub async fn collect_remote_state(client: &Client, remote_dir: &str) -> Result<FileSystemState> {
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
            }
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
            "." => {} // 忽略当前目录符号
            ".." => {
                // 回到上一级目录
                if !components.is_empty() {
                    components.pop();
                }
            }
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use chrono::Utc;
    use tempfile::{tempdir};
    use tokio::fs;
    use std::path::PathBuf;
    use dirs; // 确保引入 dirs crate
    use anyhow::{Result, anyhow}; // 确保引入 Result 和 anyhow

    // 从配置文件读取WebDAV配置
    // 注意：这个函数是异步的
    async fn read_webdav_config() -> Option<(String, String, String)> {
        // 获取用户主目录
        let host = "https://webdav-1690957.pd1.123pan.cn/webdav/webdav";
        let username = "user";
        let password = "pass";

        // 确保所有必要的配置都存在
        Some((host.to_string(), username.to_string(), password.to_string()))
    }

    // 测试路径规范化函数 (不需要修改)
    #[test]
    fn test_normalize_path() {
        assert_eq!(normalize_path("test"), "/test");
        assert_eq!(normalize_path("/test"), "/test");
        assert_eq!(normalize_path("/test/"), "/test");
        assert_eq!(normalize_path("test/"), "/test");
        assert_eq!(normalize_path("/test//path"), "/test/path");
        assert_eq!(normalize_path("//test///path//"), "/test/path");
        assert_eq!(normalize_path("\\test\\path"), "/test/path");
        assert_eq!(normalize_path("/test\\path"), "/test/path");
        assert_eq!(normalize_path("/test/./path"), "/test/path");
        assert_eq!(normalize_path("/test/../path"), "/path");
        assert_eq!(normalize_path("/test/path/.."), "/test");
        assert_eq!(normalize_path("/test/path/../.."), "/");
        assert_eq!(normalize_path("/test/path/../../other"), "/other");
        assert_eq!(normalize_path("/test//./path/..//other/./"), "/test/other");
        assert_eq!(normalize_path("\\test\\..\\path\\.\\..\\other"), "/other");
    }

    // 以下测试需要实际的WebDAV服务器和配置文件 (~/webdav.txt)，默认被忽略
    // 要运行这些测试，请使用: cargo test -- --ignored

    #[tokio::test]
    async fn test_create_client() -> Result<()> { // 确保返回 Result<()>
        // 从配置文件读取配置
        let (host, username, password) = read_webdav_config().await
            .expect("WebDAV config not found or invalid. Create ~/webdav.txt with host, username, password.");

        // 使用读取的配置创建客户端
        let result = create_client(&host, &username, &password).await;
        assert!(result.is_ok(), "Failed to create client: {:?}", result.err());
        Ok(()) // 测试成功返回 Ok(())
    }

    #[tokio::test]
    async fn test_test_connection() -> Result<()> { // 确保返回 Result<()>
        // 从配置文件读取配置
        let (host, username, password) = read_webdav_config().await
            .expect("WebDAV config not found or invalid. Create ~/webdav.txt with host, username, password.");

        // 测试连接到配置文件中的主机
        let result = test_connection(&host, &username, &password).await;
        assert!(result.is_ok(), "Failed to connect to WebDAV host {}: {:?}", host, result.err());

        // 测试连接到不存在的主机或路径 (保持硬编码，因为这是测试函数本身的错误处理)
        // 注意：这个硬编码的路径可能会随着你的WebDAV服务器设置而需要调整
        // 它的目的是测试 test_connection 在连接失败时的行为
        let wrong_host_or_path = format!("{}/nonexistent_test_path_12345", host); // 使用配置的主机，但一个不存在的路径
        eprintln!("测试连接到不存在的路径: {}", wrong_host_or_path); // 打印测试信息
        let result = test_connection(&wrong_host_or_path, &username, &password).await;
        assert!(result.is_err(), "Connection to non-existent path should fail");
        // 可以进一步检查错误消息是否符合预期

        Ok(()) // 测试成功返回 Ok(())
    }

    #[tokio::test]
    async fn test_ensure_remote_dir_exists() -> Result<()> { // 确保返回 Result<()>
        // 从配置文件读取配置
        let (host, username, password) = read_webdav_config().await
            .expect("WebDAV config not found or invalid. Create ~/webdav.txt with host, username, password.");

        let client = create_client(&host, &username, &password).await?; // 使用问号传播错误

        // 使用一个随机名称的目录，避免冲突
        let test_dir_base = format!("/test_ensure_dir_{}", Utc::now().format("%Y%m%d%H%M%S"));

        // 测试创建单层目录
        let test_dir_single = format!("{}/single", test_dir_base);
        eprintln!("测试创建单层目录: {}", test_dir_single);
        let result = ensure_remote_dir_exists(&client, &test_dir_single).await;
        assert!(result.is_ok(), "Failed to create single directory: {:?}", result.err());

        // 测试创建嵌套目录
        let nested_dir = format!("{}/nested/deep", test_dir_base);
        eprintln!("测试创建嵌套目录: {}", nested_dir);
        let result = ensure_remote_dir_exists(&client, &nested_dir).await;
        assert!(result.is_ok(), "Failed to create nested directory: {:?}", result.err());

        // TODO: 添加清理逻辑，删除创建的目录及其内容
        // WebDAV DELETE 请求通常可以递归删除
        // client.delete(&test_dir_base).await?; // 你需要一个 delete 方法

        Ok(()) // 测试成功返回 Ok(())
    }

    #[tokio::test]
    async fn test_upload_and_download_file() -> Result<()> { // 确保返回 Result<()>
        // 创建临时目录和文件
        let temp_dir = tempdir()?;
        let file_path = temp_dir.path().join("test_file_upload.txt");
        let content = "Hello, WebDAV Upload and Download Test!";

        {
            // 使用 std::fs::File::create 和 write_all 写入文件
            let mut file = std::fs::File::create(&file_path)?; // 使用问号传播错误
            file.write_all(content.as_bytes())?; // 使用问号传播错误
        }

        // 从配置文件读取配置
        let (host, username, password) = read_webdav_config().await
            .expect("WebDAV config not found or invalid. Create ~/webdav.txt with host, username, password.");

        let client = create_client(&host, &username, &password).await?; // 使用问号传播错误

        // 使用一个随机名称的远程路径
        let remote_path = format!("/test_upload_file_{}.txt", Utc::now().format("%Y%m%d%H%M%S"));
         eprintln!("测试上传文件到远程路径: {}", remote_path);

        // 上传文件
        let result = upload_file(&client, &file_path, &remote_path).await;
        assert!(result.is_ok(), "Failed to upload file: {:?}", result.err());

        // 下载文件到另一个临时位置
        let download_path = temp_dir.path().join("downloaded_test_file.txt");
         eprintln!("测试从远程路径下载文件到: {}", download_path.display());

        let result = download_file(&client, &remote_path, &download_path).await;
        assert!(result.is_ok(), "Failed to download file: {:?}", result.err());

        // 验证内容一致
        let downloaded_content = fs::read_to_string(&download_path).await?; // 使用问号传播错误
        assert_eq!(downloaded_content, content, "Downloaded content mismatch");

        // TODO: 添加清理逻辑，删除上传的文件
        // client.delete(&remote_path).await?;

        Ok(()) // 测试成功返回 Ok(())
    }

    #[tokio::test]
    async fn test_collect_remote_state() -> Result<()> { // 确保返回 Result<()>
        // 从配置文件读取配置
        let (host, username, password) = read_webdav_config().await
            .expect("WebDAV config not found or invalid. Create ~/webdav.txt with host, username, password.");

        let client = create_client(&host, &username, &password).await?; // 使用问号传播错误

        // 测试收集远程根目录状态
        let remote_dir = "/";
        eprintln!("测试收集远程目录状态: {}", remote_dir);
        let result = collect_remote_state(&client, remote_dir).await;
        assert!(result.is_ok(), "Failed to collect remote state for {}: {:?}", remote_dir, result.err());

        if let Ok(state) = result {
            // 验证状态包含条目 (假设根目录非空)
            eprintln!(
                "收集到 {} 个条目 ({} 文件, {} 目录) 从 {}",
                state.entry_count(),
                state.file_count(),
                state.directory_count(),
                remote_dir
            );
            // 你可以在这里添加更详细的断言，例如检查是否存在某个预期的目录或文件
        }

        // TODO: 可以添加测试，收集一个已知的小型测试目录的状态，并精确验证条目数量和类型

        Ok(()) // 测试成功返回 Ok(())
    }

     // TODO: 添加一个清理测试函数，用于删除所有由测试创建的临时文件和目录
     // #[tokio::test]
     // #[ignore]
     // async fn cleanup_test_artifacts() -> Result<()> {
     //    // 读取配置
     //    let (host, username, password) = read_webdav_config().await
     //       .expect("WebDAV config not found or invalid. Create ~/webdav.txt with host, username, password.");
     //
     //    let client = create_client(&host, &username, &password).await?;
     //
     //    // 删除 test_ensure_dir_* 和 test_upload_file_*
     //    // 这需要列出根目录，找到符合模式的条目，然后逐一删除
     //    // 或者如果你知道所有的临时目录和文件名称，直接删除它们
     //    // client.delete("/test_ensure_dir_...").await?;
     //    // client.delete("/test_upload_file_...").await?;
     //
     //    Ok(())
     // }
}