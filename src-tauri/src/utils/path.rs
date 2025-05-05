use anyhow::Result;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use tauri::Manager;

const APP_NAME: &str = "ToDoPulse";

static APP_PATHS: OnceLock<AppPaths> = OnceLock::new();

pub struct AppPaths {
    data_dir: PathBuf,
    config_dir: PathBuf,
    log_dir: PathBuf,
    export_dir: PathBuf,
}

impl AppPaths {
    pub fn init(app: &tauri::AppHandle) -> Result<()> {
        if APP_PATHS.get().is_some() {
            return Ok(());
        }

        let data_dir = app.path().data_dir()?.join(APP_NAME);
        let config_dir = app.path().config_dir()?.join(APP_NAME);
        let log_dir = app.path().app_log_dir()?.join(APP_NAME);
        
        // 使用回退策略来获取文档目录
        let export_dir = match app.path().document_dir() {
            Ok(path) => path.join(APP_NAME),
            Err(_) => {
                let fallback_path = app.path().home_dir()?.join(APP_NAME);
                eprintln!("Failed to get document directory, using home directory as fallback: {:?}", fallback_path);
                fallback_path
            }
        };

        // 其余代码保持不变
        ensure_dir_exists(&data_dir)?;
        ensure_dir_exists(&config_dir)?;
        ensure_dir_exists(&log_dir)?;
        ensure_dir_exists(&export_dir)?;

        let paths = AppPaths {
            data_dir,
            config_dir,
            log_dir,
            export_dir,
        };

        match APP_PATHS.set(paths) {
            Ok(_) => Ok(()),
            Err(_) => Err(anyhow::anyhow!("Failed to initialize app paths")),
        }
    }

    pub fn get() -> &'static AppPaths {
        APP_PATHS
            .get()
            .expect("AppPaths not initialized. Call AppPaths::init() first")
    }

    pub fn data_dir() -> &'static PathBuf {
        &Self::get().data_dir
    }

    pub fn config_dir() -> &'static PathBuf {
        &Self::get().config_dir
    }

    pub fn log_dir() -> &'static PathBuf {
        &Self::get().log_dir
    }

    pub fn export_dir() -> &'static PathBuf {
        &Self::get().export_dir
    }
}

fn ensure_dir_exists(path: &Path) -> Result<()> {
    if !path.exists() {
        std::fs::create_dir_all(path)?;
    }
    Ok(())
}
