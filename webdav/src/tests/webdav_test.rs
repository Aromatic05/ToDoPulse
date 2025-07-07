use crate::function::sync::webdav::{
    create_client, test_connection, ensure_remote_dir_exists, upload_file, download_file,
    collect_remote_state, normalize_path,
};

use std::io::Write;
use chrono::Utc;
use tempfile::tempdir;
use tokio::fs;
use anyhow::Result;

// 从配置文件读取WebDAV配置
// 注意：这个函数是异步的
async fn read_webdav_config() -> Option<(String, String, String)> {
    // 获取用户主目录
    let host = "https://webdav-1690957.pd1.123pan.cn/webdav/webdav";
    let username = "19852708075";
    let password = "1ux6el41000d9v0ryy3ciwkvgbj3jmr3";

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
async fn test_create_client() -> Result<()> {
    // 从配置文件读取配置
    let (host, username, password) = read_webdav_config().await
        .expect("WebDAV config not found or invalid. Create ~/webdav.txt with host, username, password.");

    // 使用读取的配置创建客户端
    let result = create_client(&host, &username, &password).await;
    assert!(result.is_ok(), "Failed to create client: {:?}", result.err());
    Ok(())
}

#[tokio::test]
async fn test_test_connection() -> Result<()> {
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

    Ok(())
}

#[tokio::test]
async fn test_ensure_remote_dir_exists() -> Result<()> {
    // 从配置文件读取配置
    let (host, username, password) = read_webdav_config().await
        .expect("WebDAV config not found or invalid. Create ~/webdav.txt with host, username, password.");

    let client = create_client(&host, &username, &password).await?;

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

    Ok(())
}

#[tokio::test]
async fn test_upload_and_download_file() -> Result<()> {
    // 创建临时目录和文件
    let temp_dir = tempdir()?;
    let file_path = temp_dir.path().join("test_file_upload.txt");
    let content = "Hello, WebDAV Upload and Download Test!";

    {
        // 使用 std::fs::File::create 和 write_all 写入文件
        let mut file = std::fs::File::create(&file_path)?;
        file.write_all(content.as_bytes())?;
    }

    // 从配置文件读取配置
    let (host, username, password) = read_webdav_config().await
        .expect("WebDAV config not found or invalid. Create ~/webdav.txt with host, username, password.");

    let client = create_client(&host, &username, &password).await?;

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
    let downloaded_content = fs::read_to_string(&download_path).await?;
    assert_eq!(downloaded_content, content, "Downloaded content mismatch");

    // TODO: 添加清理逻辑，删除上传的文件
    // client.delete(&remote_path).await?;

    Ok(())
}

#[tokio::test]
async fn test_collect_remote_state() -> Result<()> {
    // 从配置文件读取配置
    let (host, username, password) = read_webdav_config().await
        .expect("WebDAV config not found or invalid. Create ~/webdav.txt with host, username, password.");

    let client = create_client(&host, &username, &password).await?;

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

    Ok(())
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