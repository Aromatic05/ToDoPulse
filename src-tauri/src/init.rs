use crate::entity::{App, Storage, StorageState};
use crate::function::{notify, sync};
use crate::utils::config;
use crate::utils::logs;
use crate::utils::manager::tasker;
use crate::utils::AppPaths;
use anyhow::Result;
use tauri::Manager;
use tokio::sync::Mutex;

/// 应用程序初始化
pub async fn initialize_app(app: &tauri::App) -> Result<()> {
    // 初始化应用路径
    AppPaths::init(app.handle())?;

    // 初始化日志
    logs::init_log();

    // 设置通知
    notify::setup().await;

    // 读取配置文件
    config::parse()?;

    // 初始化任务管理器
    tasker::init_task_manager();

    // 创建应用实例和存储
    let app_instance = App::new(app.handle());
    let storage = Storage::new()?;
    app.manage(StorageState(Mutex::new(storage), Mutex::new(app_instance)));

    // 注册同步功能命令
    sync::register_sync_commands(app)?;

    // 解析配置
    if let Err(e) = crate::utils::config::parse() {
        eprintln!("Error parsing config: {}", e);
    }

    Ok(())
}
