use anyhow::Result;
use once_cell::sync::Lazy;
use serde::Deserialize;
use std::fs;
use std::sync::Mutex;
use std::path::{Path, PathBuf};
use toml;

use crate::utils::path::AppPaths;

const CONFIG_FILE: &str = "config.toml";

#[derive(Deserialize, Clone, Debug)]
#[allow(dead_code)]
pub struct WebDav {
    pub enabled: bool,
    pub host: String,
    pub username: String,
    pub password: String,
    pub remote_dir: String,
    pub sync_interval: u64,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct Theme {
    color: String,
}
#[derive(Deserialize)]
#[allow(dead_code)]
struct Model {
    switch: bool,
    name: String,
    tokens: String,
}
#[derive(Deserialize)]
#[allow(dead_code)]
struct Info {
    switch: bool,
    time: Option<Vec<String>>,
}
#[derive(Deserialize)]
#[allow(dead_code)]
struct Config {
    theme: Theme,
    info: Info,
    model: Model,
    webdav: WebDav,
}

static CONFIG: Lazy<Mutex<Option<Config>>> = Lazy::new(|| Mutex::new(None));

pub fn parse() -> Result<()> {
    parse_with_path::<PathBuf>(None)
}

pub fn parse_with_path<P: AsRef<Path>>(custom_path: Option<P>) -> Result<()> {
    let config_path = match custom_path {
        Some(path) => {
            let path_buf = PathBuf::from(path.as_ref());
            if path_buf.is_dir() {
                // 如果是目录，则在目录下查找配置文件
                path_buf.join(CONFIG_FILE)
            } else {
                // 如果已经是文件路径，则直接使用
                path_buf
            }
        },
        None => AppPaths::config_dir().join(CONFIG_FILE),
    };

    if !config_path.exists() {
        fs::create_dir_all(config_path.parent().unwrap())?;
        fs::write(
            &config_path,
            r#"
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
          "#,
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

pub fn use_info() -> bool {
    let config_lock = CONFIG.lock().unwrap();
    if let Some(config) = &*config_lock {
        return config.info.switch;
    }
    false
}

pub fn info_time() -> Vec<String> {
    let config_lock = CONFIG.lock().unwrap();
    if let Some(config) = &*config_lock {
        if let Some(time) = &config.info.time {
            return time.clone();
        }
    }
    vec![]
}

/// 获取WebDAV配置
pub fn get_webdav_config() -> Result<WebDav> {
    let config_lock = CONFIG.lock().unwrap();
    if let Some(config) = &*config_lock {
        // 返回WebDav结构体的克隆值
        return Ok(config.webdav.clone());
    }
    Err(anyhow::anyhow!("WebDAV configuration not found").into())
}

pub fn get_sync_interval() -> u64 {
    let config_lock = CONFIG.lock().unwrap();
    if let Some(config) = &*config_lock {
        return config.webdav.sync_interval;
    }
    30 // 默认30分钟
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
        switch = false
        name = "test-model"
        tokens = "1024"
        "#;
        
        fs::create_dir_all(&config_dir).unwrap();
        fs::write(config_dir.join(CONFIG_FILE), config_content).unwrap();
        
        // 使用自定义路径解析配置
        parse_with_path(Some(&config_dir)).unwrap();
        
        assert_eq!(use_llm(), false);
        assert_eq!(get_api_key().unwrap(), "test-model");
        assert_eq!(use_info(), false);
        assert_eq!(info_time(), vec!["0 10 * * *"]);
    }

    #[test]
    fn test_get_webdav_config() {
        let config_dir = setup_test_env();

        let config_content = r#"
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
        parse_with_path(Some(&config_dir)).unwrap();
        
        let webdav_config = get_webdav_config().unwrap();

        print!("webdav_config: {:?}", webdav_config);
        
        assert_eq!(webdav_config.enabled, true);
        assert_eq!(webdav_config.host, "https://webdav-1690957.pd1.123pan.cn/webdav/webdav");
    }
}