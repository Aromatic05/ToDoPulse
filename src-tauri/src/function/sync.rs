use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::{AppHandle, Manager, State};
use tokio::sync::Mutex;

use crate::utils::config::{self};

// 导入同步子模块
pub mod diff;
pub mod model;
pub mod state;
pub mod sync_operations;
#[cfg(test)]
pub mod tests;
pub mod webdav;

// 同步状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncStatus {
    /// 空闲状态
    Idle,
    /// 正在同步
    Syncing,
    /// 同步出错
    Error(String),
    /// 上次同步时间
    LastSyncTime(chrono::DateTime<chrono::Utc>),
}

// 同步状态管理器接口
pub trait SyncStateController {
    fn get_status(&self) -> SyncStatus;
    fn set_status(&mut self, status: SyncStatus);
}

// 同步状态管理器
pub struct SyncStateManager {
    status: SyncStatus,
}

impl SyncStateManager {
    // 创建新的状态管理器
    pub fn new() -> Self {
        Self {
            status: SyncStatus::Idle,
        }
    }
}

impl SyncStateController for SyncStateManager {
    // 获取当前同步状态
    fn get_status(&self) -> SyncStatus {
        self.status.clone()
    }

    // 设置同步状态
    fn set_status(&mut self, status: SyncStatus) {
        self.status = status;
    }
}

// 供Tauri使用的状态包装
pub struct SyncState(pub Arc<Mutex<SyncStateManager>>);

impl SyncState {
    // 创建新实例
    pub fn new(manager: SyncStateManager) -> Self {
        Self(Arc::new(Mutex::new(manager)))
    }

    // 获取状态的便捷方法
    pub async fn get_status(&self) -> SyncStatus {
        let manager = self.0.lock().await;
        manager.get_status()
    }

    // 设置状态的便捷方法
    pub async fn set_status(&self, status: SyncStatus) {
        let mut manager = self.0.lock().await;
        manager.set_status(status);
    }

    // 检查是否正在同步
    pub async fn is_syncing(&self) -> bool {
        matches!(self.get_status().await, SyncStatus::Syncing)
    }
}

// 注册同步功能的Tauri命令
pub fn register_sync_commands(app: &tauri::App) -> Result<()> {
    // 创建并注册同步状态管理器
    let sync_state = SyncState::new(SyncStateManager::new());
    app.manage(sync_state);

    Ok(())
}

pub async fn sync(state: State<'_, SyncState>) -> Result<(), String> {
    let webdav_config = match config::WebDav::load() {
        Ok(config) => config,
        Err(e) => {
            log::error!("获取WebDAV配置失败: {}", e);
            return Err(format!("获取WebDAV配置失败: {}", e));
        }
    };
    if !webdav_config.enabled {
        log::warn!("WebDAV同步未启用");
        return Err("WebDAV同步未启用".to_string());
    }

    // 检查是否已经在同步中
    if state.is_syncing().await {
        log::warn!("同步操作正在进行中");
        return Err("同步操作正在进行中".to_string());
    }

    // 更新状态为同步中
    state.set_status(SyncStatus::Syncing).await;

    // 执行同步（在后台）
    let state_clone = state.inner().0.clone();
    tokio::spawn(async move {
        let result = sync_operations::perform_sync().await;

        // 更新同步状态
        let mut manager = state_clone.lock().await;
        match result {
            Ok(session) => {
                if session.status == model::SyncSessionStatus::Completed {
                    // 同步成功
                    let now = chrono::Utc::now();
                    manager.set_status(SyncStatus::LastSyncTime(now));
                    log::info!("数据同步完成");
                } else if let Some(error) = session.error {
                    // 同步失败
                    manager.set_status(SyncStatus::Error(error.clone()));
                    log::error!("同步失败: {}", error);
                }
            }
            Err(e) => {
                let error_msg = format!("同步失败: {}", e);
                manager.set_status(SyncStatus::Error(error_msg.clone()));
                log::error!("{}", error_msg);
            }
        }
    });

    Ok(())
}

/// Initiates an immediate synchronization with the WebDAV server
///
/// Starts a WebDAV synchronization process in the background. The function
/// returns immediately while the sync operation continues asynchronously.
/// Only one sync operation can run at a time.
///
/// # Parameters
/// * `_app_handle` - Handle to the Tauri application
/// * `state` - Application state containing sync status information
///
/// # Returns
/// * `Result<(), String>` - Success or an error message if sync couldn't be started
#[tauri::command]
pub async fn sync_now(_app_handle: AppHandle, state: State<'_, SyncState>) -> Result<(), String> {
    log::info!("开始手动同步数据");
    return sync(state).await.map_err(|e| {
        log::error!("手动同步失败: {}", e);
        format!("手动同步失败: {}", e)
    });
}

/// Gets the current synchronization status
///
/// Returns the current state of WebDAV synchronization, which can be:
/// - Idle (not currently syncing)
/// - Syncing (sync operation in progress)
/// - Error (last sync operation failed with error message)
/// - LastSyncTime (last successful sync timestamp)
///
/// # Parameters
/// * `_app_handle` - Handle to the Tauri application
/// * `state` - Application state containing sync status information
///
/// # Returns
/// * `Result<SyncStatus, String>` - Current sync status or error
#[tauri::command]
pub async fn get_sync_status(
    _app_handle: AppHandle,
    state: State<'_, SyncState>,
) -> Result<SyncStatus, String> {
    Ok(state.get_status().await)
}

/// Tests the connection to a WebDAV server
///
/// Attempts to connect to the specified WebDAV server with the provided credentials.
/// This is used to validate WebDAV settings before enabling sync functionality.
///
/// # Parameters
/// * `host` - WebDAV server URL (e.g., "https://nextcloud.example.com/remote.php/dav/files/username/")
/// * `username` - WebDAV username for authentication
/// * `password` - WebDAV password for authentication
///
/// # Returns
/// * `Result<bool, String>` - true if connection successful, or error message
#[tauri::command]
pub async fn test_webdav_connection(
    host: String,
    username: String,
    password: String,
) -> Result<bool, String> {
    log::info!("测试WebDAV连接: {}", host);

    match webdav::test_connection(&host, &username, &password).await {
        Ok(result) => Ok(result),
        Err(e) => {
            log::error!("WebDAV连接测试失败: {}", e);
            Err(format!("连接失败: {}", e))
        }
    }
}
