use anyhow::Result;
use once_cell::sync::Lazy;
use serde::Deserialize;
use std::fs;
use std::sync::Mutex;
use toml;

use crate::utils::path::AppPaths;

const CONFIG_FILE: &str = "config.toml";

#[derive(Deserialize)]
struct Theme {
    _color: String,
}
#[derive(Deserialize)]
struct Model {
    switch: bool,
    name: String,
    _tokens: String,
}
#[derive(Deserialize)]
struct Info {
    _switch: bool,
    _time: Option<Vec<String>>,
}
#[derive(Deserialize)]
struct Config {
    _theme: Theme,
    _info: Info,
    model: Model,
}

static CONFIG: Lazy<Mutex<Option<Config>>> = Lazy::new(|| Mutex::new(None));

pub fn parse() -> Result<(), String> {
    let config_path = AppPaths::config_dir().join(CONFIG_FILE);
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
        return config._info._switch;
    }
    false
}
