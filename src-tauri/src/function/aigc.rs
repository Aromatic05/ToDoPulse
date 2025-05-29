use anyhow::Result;
use log::{error};
use std::sync::Mutex;
use tauri::{AppHandle, State, Emitter};
use once_cell::sync::Lazy;

use crate::entity::{get_tags, StorageState};
use crate::utils::config::Model;

// 用于存储生成的标签的全局变量
static GENERATED_TAGS: Lazy<Mutex<Option<Vec<String>>>> = Lazy::new(|| Mutex::new(None));

#[tauri::command]
pub async fn gen_tag(
    app: AppHandle,
    state: State<'_, StorageState>,
    title: &str,
) -> Result<Option<Vec<String>>> {
    let llm_config = Model::load().unwrap_or_default();
    let use_llm = llm_config.switch;
    if !use_llm {
        return Ok(None);
    }

    // 获取所有可用标签
    let available_tags = get_tags(state)
        .await?
        .iter()
        .map(|tag| tag.name.clone())
        .collect::<Vec<String>>();

    // 调用前端的 AigcService.generateTags 方法
    // 清空之前的标签
    *GENERATED_TAGS.lock().unwrap() = None;

    // 发送事件到前端
    app.emit("aigc:generate-tags", (title, available_tags))?;

    // 等待前端返回标签(最多等待10秒)
    let start_time = std::time::Instant::now();
    while start_time.elapsed() < std::time::Duration::from_secs(10) {
        if let Some(tags) = GENERATED_TAGS.lock().unwrap().take() {
            return Ok(Some(tags));
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }

    error!("Timeout waiting for tags from frontend");
    Ok(None)
}

/// 接收前端生成的标签
#[tauri::command]
pub async fn receive_generated_tags(tags: Vec<String>) -> tauri::Result<()> {
    *GENERATED_TAGS.lock().unwrap() = Some(tags);
    Ok(())
}
