use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::sync::OnceLock;
use tokio::sync::Mutex;
use tauri::{AppHandle, Manager};

use crate::utils::config::{self, WebDav};

// 导入同步子模块
pub mod webdav;
pub mod model;
pub mod state;
pub mod diff;
pub mod sync_operations;

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

// 全局同步状态管理器
static SYNC_STATE_MANAGER: OnceLock<Arc<Mutex<SyncStateManager>>> = OnceLock::new();

// 同步状态管理器
pub struct SyncStateManager {
    status: SyncStatus,
}

impl SyncStateManager {
    // 获取实例（单例模式）
    pub fn get_instance() -> Arc<Mutex<SyncStateManager>> {
        SYNC_STATE_MANAGER.get_or_init(|| {
            Arc::new(Mutex::new(SyncStateManager {
                status: SyncStatus::Idle,
            }))
        }).clone()
    }
    
    // 获取当前同步状态
    pub fn get_status(&self) -> SyncStatus {
        self.status.clone()
    }
    
    // 设置同步状态
    pub fn set_status(&mut self, status: SyncStatus) {
        self.status = status;
    }
}

// 供Tauri使用的状态包装
pub struct SyncState(pub Arc<Mutex<SyncStateManager>>);

// 注册同步功能的Tauri命令
pub fn register_sync_commands(app: &mut tauri::App) -> Result<()> {
    // 初始化同步状态管理器并注册为应用状态
    let sync_manager = SyncStateManager::get_instance();
    app.manage(SyncState(sync_manager));
    
    Ok(())
}

/// Tauri命令 - 立即同步
/// 执行WebDAV文件夹同步操作
#[tauri::command]
pub async fn sync_now(_app_handle: AppHandle) -> Result<(), String> {
    log::info!("开始手动同步数据");
        
    // 检查WebDAV是否启用
    let webdav_config = match config::get_webdav_config() {
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
    
    // 获取同步状态管理器
    let sync_manager = SyncStateManager::get_instance();
    let mut manager = sync_manager.lock().await;
    
    // 检查是否已经在同步中
    if let SyncStatus::Syncing = manager.get_status() {
        log::warn!("同步操作正在进行中");
        return Err("同步操作正在进行中".to_string());
    }
    
    // 更新状态为同步中
    manager.set_status(SyncStatus::Syncing);
    
    // 执行同步（在后台）
    let sync_manager_clone = sync_manager.clone();
    tokio::spawn(async move {
        let result = sync_operations::perform_sync().await;
        
        // 更新同步状态
        let mut manager = sync_manager_clone.lock().await;
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
            },
            Err(e) => {
                let error_msg = format!("同步失败: {}", e);
                manager.set_status(SyncStatus::Error(error_msg.clone()));
                log::error!("{}", error_msg);
            }
        }
    });
    
    Ok(())
}

/// Tauri命令 - 获取同步状态
/// 返回当前的WebDAV同步状态
#[tauri::command]
pub async fn get_sync_status(_app_handle: AppHandle) -> Result<SyncStatus, String> {
    let sync_manager = SyncStateManager::get_instance();
    let manager = sync_manager.lock().await;
    
    Ok(manager.get_status())
}

/// Tauri命令 - 测试WebDAV连接
/// 测试是否能够连接到WebDAV服务器
#[tauri::command]
pub async fn test_webdav_connection(
    host: String,
    username: String,
    password: String
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

/// Tauri命令 - 更新WebDAV同步配置
/// 保存WebDAV连接配置
#[tauri::command]
pub async fn update_sync_config(
    enabled: bool,
    host: String,
    username: String,
    password: String,
    remote_dir: String
) -> Result<(), String> {
    log::info!("更新WebDAV同步配置");
    
    // 创建WebDAV配置
    let webdav = WebDav {
        enabled,
        host,
        username,
        password,
        remote_dir,
        sync_interval: 30, // 使用默认值
    };
    
    // 由于当前没有实现保存WebDAV配置的函数，返回临时错误
    // TODO: 实现保存WebDAV配置的功能
    log::error!("保存WebDAV配置功能尚未实现");
    Err("保存WebDAV配置功能尚未实现，请修改config.toml文件".to_string())
}
