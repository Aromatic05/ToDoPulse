use anyhow::Result;
use once_cell::sync::Lazy;
use serde::Deserialize;
use std::fs;
use std::sync::Mutex;
use toml;

use crate::utils::path::AppPaths;

const CONFIG_FILE: &str = "config.toml";
const BACK_UP_CONFIG_FILE: &str = "config_backup.toml";

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
    prompt: String,
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
}

static CONFIG: Lazy<Mutex<Option<Config>>> = Lazy::new(|| Mutex::new(None));

pub fn parse() -> Result<(), String> {
    let config_path = AppPaths::config_dir().join(CONFIG_FILE);
    let backup_path = AppPaths::config_dir().join(BACK_UP_CONFIG_FILE);
    if !config_path.exists() {
        fs::create_dir_all(config_path.parent().unwrap()).map_err(|e| e.to_string())?;
        fs::write(
            &config_path,
            r#"
          [theme]
          color = "blue"
          [info]
          switch = true
          time = ["08:00", "12:00", "18:00"]
          [model]
          switch = false
          name = "gpt-3.5-turbo"
          tokens = "4096"
          "#,
        )
        .map_err(|e| e.to_string())?;
    }
    let config_str = fs::read_to_string(config_path).map_err(|e| e.to_string())?;
    let config: Config = toml::from_str(&config_str).map_err(|e| e.to_string())?;

    let mut config_lock = CONFIG.lock().unwrap();
    *config_lock = Some(config);
    Ok(())
}

pub fn get_api_key() -> Result<String, String> {
    let config_lock = CONFIG.lock().unwrap();
    if let Some(config) = &*config_lock {
        return Ok(config.model.name.clone());
    }
    Err("API key not found".to_string())
}

pub fn use_llm() -> bool {
    let config_lock = CONFIG.lock().unwrap();
    if let Some(config) = &*config_lock {
        return config.model.switch;
    }
    false
}