use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::sync::OnceLock;
use tokio::sync::Mutex;
use tauri::{AppHandle, Manager};

use crate::utils::path::AppPaths;

// 导入WebDAV同步子模块
pub mod webdav;

// WebDAV同步配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebDavConfig {
    pub enabled: bool,
    pub host: String,
    pub username: String,
    pub password: String,
    pub remote_dir: String,
    pub sync_interval: u64, // 同步间隔（分钟）
}

impl Default for WebDavConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            host: String::new(),
            username: String::new(),
            password: String::new(),
            remote_dir: "/ToDoPulse".to_string(),
            sync_interval: 30,
        }
    }
}

// 同步状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncStatus {
    Idle,
    Syncing,
    Error(String),
    LastSyncTime(chrono::DateTime<chrono::Utc>),
}

// 全局同步管理器
static SYNC_MANAGER: OnceLock<Arc<Mutex<SyncManager>>> = OnceLock::new();

// 同步管理器
pub struct SyncManager {
    config: WebDavConfig,
    status: Arc<Mutex<SyncStatus>>,
    app: tauri::AppHandle,
}

// 状态管理器类型
pub struct SyncState(pub Arc<Mutex<SyncManager>>);

impl SyncManager {
    // 获取同步管理器实例（单例模式）
    pub fn get_instance(app_handle: tauri::AppHandle) -> Arc<Mutex<SyncManager>> {
        SYNC_MANAGER.get_or_init(|| {
            let config = WebDavConfig::default();
            Arc::new(Mutex::new(SyncManager::new_internal(app_handle.clone(), config)))
        }).clone()
    }
    
    // 内部创建新的同步管理器实例
    fn new_internal(app: tauri::AppHandle, config: WebDavConfig) -> Self {
        Self {
            app,
            config,
            status: Arc::new(Mutex::new(SyncStatus::Idle)),
        }
    }

    // 获取当前同步状态
    pub async fn get_status(&self) -> SyncStatus {
        self.status.lock().await.clone()
    }

    // 开始手动同步
    pub async fn sync_now(&self) -> Result<()> {
        log::info!("开始手动同步数据");
        
        if !self.config.enabled {
            log::warn!("WebDAV同步未启用");
            return Ok(());
        }

        // 更新状态为同步中
        *self.status.lock().await = SyncStatus::Syncing;
        
        // 获取数据目录
        let data_dir = AppPaths::data_dir().clone();
        
        // 执行同步
        let result = webdav::sync_directory(
            &self.config.host,
            &self.config.username,
            &self.config.password,
            &data_dir,
            &self.config.remote_dir,
        ).await;
        
        // 更新同步状态
        match result {
            Ok(_) => {
                let now = chrono::Utc::now();
                *self.status.lock().await = SyncStatus::LastSyncTime(now);
                log::info!("数据同步完成");
                Ok(())
            },
            Err(e) => {
                let error_msg = format!("同步失败: {}", e);
                *self.status.lock().await = SyncStatus::Error(error_msg.clone());
                log::error!("{}", error_msg);
                Err(e)
            }
        }
    }

    // 设置新的配置
    pub fn update_config(&mut self, config: WebDavConfig) -> Result<()> {
        log::info!("更新WebDAV同步配置");
        self.config = config;
        Ok(())
    }
    
    // 测试WebDAV连接
    pub async fn test_connection(&self) -> Result<bool> {
        log::info!("测试WebDAV连接");
        
        webdav::test_connection(
            &self.config.host,
            &self.config.username,
            &self.config.password
        ).await
    }
    
    // 启动定时同步任务
    pub fn start_scheduled_sync(&self) -> Result<()> {
        if !self.config.enabled || self.config.sync_interval == 0 {
            log::info!("定时同步未启用");
            return Ok(());
        }
        
        log::info!("启动定时同步任务，间隔: {} 分钟", self.config.sync_interval);
        
        // 在实际应用中，这里应该设置一个定时器来定期执行同步
        // 由于实现定时器需要复杂的后台任务管理，这里只是框架代码
        
        Ok(())
    }
}

// 注册同步功能的命令到Tauri
pub fn register_sync_commands(app: &mut tauri::App) -> Result<()> {
    // 初始化同步管理器并注册为应用状态
    let sync_manager = SyncManager::get_instance(app.handle().clone());
    app.manage(SyncState(sync_manager));
    
    Ok(())
}

// Tauri 命令 - 测试WebDAV连接
#[tauri::command]
pub async fn test_webdav_connection(
    host: String,
    username: String,
    password: String,
    _app_handle: AppHandle,
) -> Result<bool, String> {
    // 直接使用传入的参数测试连接
    webdav::test_connection(&host, &username, &password).await
        .map_err(|e| e.to_string())
}

// Tauri 命令 - 立即同步
#[tauri::command]
pub async fn sync_now(app_handle: AppHandle) -> Result<(), String> {
    let sync_manager = SyncManager::get_instance(app_handle.clone());
    let manager = sync_manager.lock().await;
    
    manager.sync_now().await
        .map_err(|e| e.to_string())
}

// Tauri 命令 - 获取同步状态
#[tauri::command]
pub async fn get_sync_status(app_handle: AppHandle) -> Result<SyncStatus, String> {
    let sync_manager = SyncManager::get_instance(app_handle.clone());
    let manager = sync_manager.lock().await;
    
    Ok(manager.get_status().await)
}

// Tauri 命令 - 更新同步配置
#[tauri::command]
pub async fn update_sync_config(
    config: WebDavConfig,
    app_handle: AppHandle,
) -> Result<(), String> {
    let sync_manager = SyncManager::get_instance(app_handle.clone());
    let mut manager = sync_manager.lock().await;
    
    manager.update_config(config)
        .map_err(|e| e.to_string())
}