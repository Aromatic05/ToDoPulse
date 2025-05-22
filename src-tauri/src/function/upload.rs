use crate::entity::{Event, Repository, StorageState};
use anyhow::Result;
use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{Read, Write};
use std::ops::DerefMut;
use std::path::{Path, PathBuf};
use tauri::State;
use uuid::Uuid;

/// 上传文件的响应格式
#[derive(Serialize, Deserialize)]
pub struct UploadResponse {
    code: i32,
    msg: String,
    data: UploadData,
}

/// 上传文件的数据部分
#[derive(Serialize, Deserialize)]
pub struct UploadData {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    err_files: Vec<String>,
    succ_map: std::collections::HashMap<String, String>,
}

/// 处理文件上传（接收Base64编码的文件数据）
///
/// 接收Base64编码的文件数据，将其解码后保存到事件内容目录中，并返回文件URL
///
/// # 参数
/// * `state` - 应用状态，包含数据库访问权限
/// * `filename` - 原始文件名
/// * `file_data` - Base64编码的文件数据
/// * `event_id` - 事件ID，用于关联上传文件
///
/// # 返回
/// * 成功时返回上传响应对象，包含成功上传的文件映射
#[tauri::command]
pub async fn upload_file(
    state: State<'_, StorageState>,
    filename: String,
    filedata: String,
    eventid: String,
) -> Result<UploadResponse, String> {
    // 1. 获取事件信息，确定文件保存目录
    let mut guard = state.0.lock().await;
    let storage = guard.deref_mut();
    let event = match Repository::<Event>::get_by_name(storage, &eventid) {
        Ok(Some(event)) => event,
        Ok(None) => return Err(format!("事件不存在: {}", eventid)),
        Err(e) => return Err(format!("查询事件失败: {}", e)),
    };

    // 2. 提取事件内容文件路径，确定文件保存目录
    let content_path = PathBuf::from(&event.content);
    let dir_path = match content_path.parent() {
        Some(dir) => dir.to_path_buf(),
        None => return Err("无法获取事件内容目录".to_string()),
    };

    // 3. 解码Base64数据
    let file_content = match general_purpose::STANDARD.decode(filedata) {
        Ok(data) => data,
        Err(e) => return Err(format!("解码文件数据失败: {}", e)),
    };

    // 4. 生成文件名 (使用UUID避免冲突)
    let file_ext = Path::new(&filename)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("bin");

    let unique_filename = format!("{}.{}", Uuid::new_v4(), file_ext);
    let target_path = dir_path.join(&unique_filename);

    // 5. 写入目标文件
    match fs::File::create(&target_path).and_then(|mut file| file.write_all(&file_content)) {
        Ok(_) => {
            // 6. 返回文件URL (相对路径)
            let mut succ_map = std::collections::HashMap::new();

            // 创建相对路径 - 使用事件目录下的文件
            let file_url = format!("./{}", unique_filename);
            succ_map.insert(filename, file_url);

            Ok(UploadResponse {
                code: 0,
                msg: "上传成功".to_string(),
                data: UploadData {
                    err_files: vec![],
                    succ_map,
                },
            })
        }
        Err(e) => Err(format!("保存文件失败: {}", e)),
    }
}

/// 将网络图片下载到本地
///
/// # 参数
/// * `state` - 应用状态，包含数据库访问权限
/// * `url` - 图片URL
/// * `event_id` - 事件ID，用于关联上传文件
///
/// # 返回
/// * 成功时返回本地图片URL
#[tauri::command]
pub async fn save_remote_image(
    state: State<'_, StorageState>,
    url: String,
    eventid: String,
) -> Result<serde_json::Value, String> {
    // 1. 获取事件信息，确定文件保存目录
    let mut guard = state.0.lock().await;
    let storage = guard.deref_mut();
    let event = match Repository::<Event>::get_by_name(storage, &eventid) {
        Ok(Some(event)) => event,
        Ok(None) => return Err(format!("事件不存在: {}", eventid)),
        Err(e) => return Err(format!("查询事件失败: {}", e)),
    };

    // 2. 提取事件内容文件路径，确定文件保存目录
    let content_path = PathBuf::from(&event.content);
    let dir_path = match content_path.parent() {
        Some(dir) => dir.to_path_buf(),
        None => return Err("无法获取事件内容目录".to_string()),
    };

    // 3. 从URL提取文件扩展名
    let file_ext = url
        .split('.')
        .last()
        .and_then(|ext| {
            let ext = ext.split('?').next().unwrap_or(ext); // 移除查询参数
            if ext.len() <= 4 {
                Some(ext)
            } else {
                None
            }
        })
        .unwrap_or("jpg");

    // 4. 生成唯一文件名
    let unique_filename = format!("{}.{}", Uuid::new_v4(), file_ext);
    let file_path = dir_path.join(&unique_filename);

    // 5. 下载图片
    let response = match reqwest::get(&url).await {
        Ok(res) => res,
        Err(e) => return Err(format!("下载图片失败: {}", e)),
    };

    let bytes = match response.bytes().await {
        Ok(b) => b,
        Err(e) => return Err(format!("读取图片数据失败: {}", e)),
    };

    // 6. 保存图片到事件内容目录
    match fs::File::create(&file_path).and_then(|mut file| file.write_all(&bytes)) {
        Ok(_) => {
            // 7. 返回本地图片URL
            let local_url = format!("./{}", unique_filename);
            let response = serde_json::json!({
                "code": 0,
                "msg": "图片保存成功",
                "data": {
                    "originalURL": url,
                    "url": local_url
                }
            });
            Ok(response)
        }
        Err(e) => Err(format!("保存图片失败: {}", e)),
    }
}
