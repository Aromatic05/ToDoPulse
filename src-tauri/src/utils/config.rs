// #![allow(dead_code)]

use anyhow::Result;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::fs;
use std::sync::Mutex;
use std::path::{Path, PathBuf};
use toml;
use ts_rs::TS;

use crate::utils::path::AppPaths;
use crate::error::ErrorKind;

const CONFIG_FILE: &str = "config.toml";
const DEFAULT_CONFIG: &str = r#"
[theme]
color = "blue"
[info]
switch = false
time = ["0 12 * * *", "0 13 * * *"]
[model]
switch = false
name = "deepseek-v3"
tokens = "4096"
[webdav]
enabled = false
host = "https://example.com"
username = "user"
password = "password"
remote_dir = "/ToDoPulse"
sync_interval = 30
"#;

static CONFIG: Lazy<Mutex<Option<Config>>> = Lazy::new(|| Mutex::new(None));

#[derive(Deserialize, Serialize, Clone, TS)]
pub struct WebDav {
    pub enabled: bool,
    pub host: String,
    pub username: String,
    pub password: String,
    pub remote_dir: String,
    pub sync_interval: u64,
}

#[derive(Deserialize, Serialize, Clone, TS)]
pub struct Theme {
    color: String,
}
#[derive(Deserialize, Serialize, Clone, TS)]
pub struct Model {
    switch: bool,
    name: String,
    tokens: String,
}
#[derive(Deserialize,Serialize , Clone, TS)]
pub struct Info {
    pub switch: bool,
    pub time: Option<Vec<String>>,
}
#[derive(Deserialize, Serialize)]
struct Config {
    theme: Theme,
    info: Info,
    model: Model,
    webdav: WebDav,
}

#[derive(Clone, TS, Deserialize, Serialize)]
#[ts(export)]
pub enum ConfigField {
    Theme(Theme),
    Info(Info),
    Model(Model),
    WebDav(WebDav),
}

impl ConfigField {
    fn apply(&self, config: &mut Config) {
        match self {
            ConfigField::Theme(theme) => config.theme = theme.clone(),
            ConfigField::Info(info) => config.info = info.clone(),
            ConfigField::Model(model) => config.model = model.clone(),
            ConfigField::WebDav(webdav) => config.webdav = webdav.clone(),
        }
    }
} 

pub fn parse() -> Result<()> {
    parse_with_path::<PathBuf>(None)
}

#[tauri::command]
pub fn update_config(field: ConfigField) -> Result<(), ErrorKind> {
    let mut config_lock = CONFIG.lock().unwrap();
    if let Some(config) = &mut *config_lock {
        field.apply(config);
        write_config()?;
        log::info!("Config updated");
        Ok(())
    } else {
        log::error!("Config not found");
        Err(anyhow::anyhow!("Config not found").into())
    }
}

pub fn write_config() -> Result<()> {
    let config_path = AppPaths::config_dir().join(CONFIG_FILE);
    let config_lock = CONFIG.lock().unwrap();
    if let Some(config) = &*config_lock {
        fs::write(
            &config_path,
            toml::to_string(config)?,
        )?;
        Ok(())
    } else {
        log::error!("Config not found");
        Err(anyhow::anyhow!("no config to write"))
    }
}

pub fn parse_with_path<P: AsRef<Path>>(custom_path: Option<P>) -> Result<()> {
    let config_path = match custom_path {
        Some(path) => {
            let path_buf = PathBuf::from(path.as_ref());
            if path_buf.is_dir() {
                path_buf.join(CONFIG_FILE)
            } else {
                path_buf
            }
        },
        None => AppPaths::config_dir().join(CONFIG_FILE),
    };

    if !config_path.exists() {
        fs::create_dir_all(config_path.parent().unwrap())?;
        fs::write(
            &config_path,
            DEFAULT_CONFIG,
        )?;
    }
    let config_str = fs::read_to_string(&config_path)?;
    let config: Config = toml::from_str(&config_str)?;

    let mut config_lock = CONFIG.lock().unwrap();
    *config_lock = Some(config);
    Ok(())
}

pub fn get_api_key() -> Result<String> {
    let config_lock = CONFIG.lock().unwrap();
    if let Some(config) = &*config_lock {
        return Ok(config.model.name.clone());
    }
    Err(anyhow::anyhow!("API key not found").into())
}

pub fn use_llm() -> bool {
    let config_lock = CONFIG.lock().unwrap();
    if let Some(config) = &*config_lock {
        return config.model.switch;
    }
    false
}

pub fn info() -> Result<Info> {
    let config_lock = CONFIG.lock().unwrap();
    if let Some(config) = &*config_lock {
        return Ok(config.info.clone());
    }
    log::error!("Info config not found");
    Err(anyhow::anyhow!("Info config not found").into())
}

/// 获取WebDAV配置
pub fn get_webdav_config() -> Result<WebDav> {
    let config_lock = CONFIG.lock().unwrap();
    if let Some(config) = &*config_lock {
        // 返回WebDav结构体的克隆值
        return Ok(config.webdav.clone());
    }
    log::error!("WebDAV configuration not found");
    Err(anyhow::anyhow!("WebDAV configuration not found").into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::path::PathBuf;

    // 创建测试环境
    fn setup_test_env() -> PathBuf {
        // 创建临时目录作为配置目录
        let temp_dir = tempdir().unwrap();
        let config_dir = temp_dir.path().to_path_buf();
        config_dir
    }
    
    #[test]
    fn test_get_api_key_after_parse() {
        let config_dir = setup_test_env();
        
        // 先使用自定义路径解析，然后获取 API 密钥
        parse_with_path(Some(&config_dir)).unwrap();
        let result = get_api_key();
        
        assert!(result.is_ok());
        // 断言存在值，但不断言具体值
        assert!(!result.unwrap().is_empty());
    }
    
    #[test]
    fn test_use_llm_default() {
        // 重置 CONFIG 确保测试独立性
        *CONFIG.lock().unwrap() = None;
        
        // 获取默认值
        let result = use_llm();
        assert_eq!(result, false);
    }
    
    #[test]
    fn test_custom_config_values() {
        let config_dir = setup_test_env();
        
        // 写入自定义配置
        let config_content = r#"
        [theme]
        color = "red"
        [info]
        switch = false
        time = ["0 10 * * *"]
        [model]
        switch = true
        name = "test-model"
        tokens = "1024"
        [webdav]
        enabled = true
        host = "https://webdav-1690957.pd1.123pan.cn/webdav/webdav"
        username = "username"
        password = "passwd"
        remote_dir = "/ToDoPulse"
        sync_interval = 30
        "#;
        
        fs::create_dir_all(&config_dir).unwrap();
        fs::write(config_dir.join(CONFIG_FILE), config_content).unwrap();
        
        // 使用自定义路径解析配置
        parse_with_path(Some(&config_dir)).unwrap();
        let webdav_config = get_webdav_config().unwrap();

        assert_eq!(use_llm(), true);
        assert_eq!(get_api_key().unwrap(), "test-model");
        assert_eq!(webdav_config.enabled, true);
        assert_eq!(webdav_config.host, "https://webdav-1690957.pd1.123pan.cn/webdav/webdav");
    }
}