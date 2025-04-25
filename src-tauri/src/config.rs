use anyhow::Result;
use std::fs;
use serde::Deserialize;
use tauri::Manager;
use toml;

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

#[tauri::command]
pub fn parse(app:tauri::AppHandle) -> Result<(), String> {
    let config_path = app.path()
        .config_dir()
        .map_err(|e| e.to_string())?
        .join("config.toml");
    let config_str = fs::read_to_string(config_path).map_err(|e| e.to_string())?;
    let config: Config = toml::from_str(&config_str).map_err(|e| e.to_string())?;
    Ok(())
}
