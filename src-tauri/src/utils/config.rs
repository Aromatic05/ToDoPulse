use anyhow::Result;
use field_macro::ConfigField as F;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use toml;
use ts_rs::TS;

use crate::error::ErrorKind;
use crate::utils::path::AppPaths;

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
api = "https://api.deepseek.com/v1/chat/completions"
[webdav]
enabled = false
host = "https://example.com"
username = "user"
password = "password"
remote_dir = "/ToDoPulse"
"#;

// 解析DEFAULT_CONFIG以复用默认值
static DEFAULT_VALUES: Lazy<Config> =
    Lazy::new(|| toml::from_str(DEFAULT_CONFIG).expect("默认配置解析失败"));

static CONFIG: Lazy<Mutex<Option<Config>>> = Lazy::new(|| Mutex::new(None));

#[derive(Deserialize, Serialize, Clone, TS, F)]
pub struct WebDav {
    pub enabled: bool,
    pub host: String,
    pub username: String,
    pub password: String,
    pub remote_dir: String,
}

#[derive(Deserialize, Serialize, Clone, TS, F)]
pub struct Theme {
    color: String,
}

#[derive(Deserialize, Serialize, Clone, TS, F)]
pub struct Model {
    pub switch: bool,
    pub name: String,
    pub tokens: String,
    pub api: String,
}

#[derive(Deserialize, Serialize, Clone, TS, F)]
pub struct Info {
    pub switch: bool,
    pub time: Option<Vec<String>>,
}

#[derive(Deserialize, Serialize, Clone, TS)]
#[ts(export)]
pub struct Config {
    pub theme: Theme,
    pub info: Info,
    pub model: Model,
    pub webdav: WebDav,
}

// Config也实现Default，使用各字段的Default实现
impl Default for Config {
    fn default() -> Self {
        Self {
            theme: Theme::default(),
            info: Info::default(),
            model: Model::default(),
            webdav: WebDav::default(),
        }
    }
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

fn with_config<F, R>(f: F) -> Result<R>
where
    F: FnOnce(&Config) -> R,
{
    let config_lock = CONFIG.lock();
    match &*config_lock {
        Some(config) => Ok(f(config)),
        None => {
            log::error!("Config not found");
            Err(anyhow::anyhow!("Config not found").into())
        }
    }
}

pub fn parse() -> Result<()> {
    parse_with_path::<PathBuf>(None)
}

/// Updates a specific section of the application configuration
///
/// Modifies the specified configuration field (Theme, Info, Model, or WebDAV)
/// and writes the updated configuration to disk.
///
/// # Parameters
/// * `field` - The configuration field to update, wrapped in the appropriate ConfigField variant
///
/// # Returns
/// * `Result<(), ErrorKind>` - Success or an error if the configuration couldn't be updated
#[tauri::command]
pub fn update_config(field: ConfigField) -> Result<(), ErrorKind> {
    // 先获取配置的可变引用
    let mut config_to_update = {
        let config_lock = CONFIG.lock();
        match &*config_lock {
            Some(config) => config.clone(),
            None => {
                log::error!("Config not found");
                return Err(anyhow::anyhow!("Config not found").into());
            }
        }
    };

    // 应用更改
    field.apply(&mut config_to_update);

    // 将更新后的配置写入文件
    let config_path = AppPaths::config_dir().join(CONFIG_FILE);
    match toml::to_string(&config_to_update) {
        Ok(config_str) => match fs::write(&config_path, config_str) {
            Ok(_) => {
                // 成功写入文件后更新内存中的配置
                let mut config_lock = CONFIG.lock();
                *config_lock = Some(config_to_update);
                log::info!("Config updated and written to disk");
                Ok(())
            }
            Err(e) => {
                log::error!("Failed to write config file: {}", e);
                Err(anyhow::anyhow!("Failed to write config file").into())
            }
        },
        Err(e) => {
            log::error!("Failed to serialize config: {}", e);
            Err(anyhow::anyhow!("Failed to serialize config").into())
        }
    }
}

#[tauri::command]
pub fn get_config() -> Result<Config, ErrorKind> {
    let config = with_config(|config| config.clone())?;
    Ok(config)
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
        }
        None => AppPaths::config_dir().join(CONFIG_FILE),
    };

    if !config_path.exists() {
        fs::create_dir_all(config_path.parent().unwrap())?;
        fs::write(&config_path, DEFAULT_CONFIG)?;
    }
    let config_str = fs::read_to_string(&config_path)?;

    // 尝试解析配置
    let config: Config = match toml::from_str(&config_str) {
        Ok(config) => config,
        Err(e) => {
            log::error!("Failed to parse config file: {}", e);

            // 如果解析失败，尝试修复损坏的配置，使用默认值
            log::info!("Using default config values due to parse error");
            DEFAULT_VALUES.clone()
        }
    };

    // 检查并填充缺失的字段
    let merged_config = merge_with_defaults(config);

    // 将修复后的配置重新写入文件
    if merged_config.1 {
        log::info!("Config had missing fields, writing fixed configuration to disk");
        let config_str = toml::to_string(&merged_config.0)?;
        fs::write(&config_path, config_str)?;
    }

    let mut config_lock = CONFIG.lock();
    *config_lock = Some(merged_config.0);
    Ok(())
}

/// 将用户配置与默认配置合并，填充缺失的字段
///
/// 返回 (merged_config, was_modified)
/// - merged_config: 合并后的配置
/// - was_modified: 配置是否被修改（是否有字段被默认值填充）
fn merge_with_defaults(mut user_config: Config) -> (Config, bool) {
    let default_config = DEFAULT_VALUES.clone();
    let mut was_modified = false;

    // 使用派生宏生成的方法填充各个组件的默认值
    user_config.theme.fill_defaults_from(&default_config.theme, &mut was_modified);
    user_config.info.fill_defaults_from(&default_config.info, &mut was_modified);
    user_config.model.fill_defaults_from(&default_config.model, &mut was_modified);
    user_config.webdav.fill_defaults_from(&default_config.webdav, &mut was_modified);

    (user_config, was_modified)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::tempdir;

    // 创建测试环境
    fn setup_test_env() -> PathBuf {
        // 创建临时目录作为配置目录
        let temp_dir = tempdir().unwrap();
        let config_dir = temp_dir.path().to_path_buf();
        config_dir
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
        api = "https://api.test.com/v1/chat/completions"
        [webdav]
        enabled = true
        host = "https://webdav-1690957.pd1.123pan.cn/webdav/webdav"
        username = "username"
        password = "passwd"
        remote_dir = "/ToDoPulse"
        "#;

        fs::create_dir_all(&config_dir).unwrap();
        fs::write(config_dir.join(CONFIG_FILE), config_content).unwrap();

        // 使用自定义路径解析配置
        parse_with_path(Some(&config_dir)).unwrap();
        let webdav_config = WebDav::load().unwrap();

        assert_eq!(webdav_config.enabled, true);
        assert_eq!(
            webdav_config.host,
            "https://webdav-1690957.pd1.123pan.cn/webdav/webdav"
        );
    }
}
