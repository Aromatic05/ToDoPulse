use anyhow::Result;
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
[webdav]
enabled = false
host = "https://example.com"
username = "user"
password = "password"
remote_dir = "/ToDoPulse"
sync_interval = 30
"#;

// 解析DEFAULT_CONFIG以复用默认值
static DEFAULT_VALUES: Lazy<Config> =
    Lazy::new(|| toml::from_str(DEFAULT_CONFIG).expect("默认配置解析失败"));

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

impl Default for WebDav {
    fn default() -> Self {
        DEFAULT_VALUES.webdav.clone()
    }
}

#[derive(Deserialize, Serialize, Clone, TS)]
pub struct Theme {
    color: String,
}

impl Default for Theme {
    fn default() -> Self {
        DEFAULT_VALUES.theme.clone()
    }
}

#[derive(Deserialize, Serialize, Clone, TS)]
pub struct Model {
    pub switch: bool,
    pub name: String,
    pub tokens: String,
}

impl Default for Model {
    fn default() -> Self {
        DEFAULT_VALUES.model.clone()
    }
}

#[derive(Deserialize, Serialize, Clone, TS)]
pub struct Info {
    pub switch: bool,
    pub time: Option<Vec<String>>,
}

impl Default for Info {
    fn default() -> Self {
        DEFAULT_VALUES.info.clone()
    }
}

#[derive(Deserialize, Serialize)]
struct Config {
    theme: Theme,
    info: Info,
    model: Model,
    webdav: WebDav,
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

fn with_config_mut<F, R>(f: F) -> Result<R>
where
    F: FnOnce(&mut Config) -> R,
{
    let mut config_lock = CONFIG.lock();
    match &mut *config_lock {
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
    with_config_mut(|config| {
        field.apply(config);
        write_config()?;
        log::info!("Config updated");
        Ok(())
    })?
}

pub fn write_config() -> Result<()> {
    let config_path = AppPaths::config_dir().join(CONFIG_FILE);
    with_config(|config| {
        let config_str = toml::to_string(config)?;
        fs::write(&config_path, config_str)?;
        Ok(())
    })?
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
    let config: Config = toml::from_str(&config_str)?;

    let mut config_lock = CONFIG.lock();
    *config_lock = Some(config);
    Ok(())
}

pub fn llm_config() -> Result<Model> {
    with_config(|config| config.model.clone())
}

pub fn info() -> Result<Info> {
    with_config(|config| config.info.clone())
}

pub fn get_webdav_config() -> Result<WebDav> {
    with_config(|config| config.webdav.clone())
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

        assert_eq!(webdav_config.enabled, true);
        assert_eq!(
            webdav_config.host,
            "https://webdav-1690957.pd1.123pan.cn/webdav/webdav"
        );
    }
}
