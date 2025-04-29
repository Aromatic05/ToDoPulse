mod entity; // 核心数据实体和存储定义
mod time; // 时间处理工具

mod aigc; // AI 生成内容相关功能

mod config; // 配置管理
mod debug; // 调试工具
mod utils; // 通用工具函数

use entity::{Storage, StorageState};
use std::sync::Mutex;
use tauri::Manager;

pub use entity::{Event, List, Tag};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() -> std::io::Result<()> {
    tauri::Builder::default()
        .setup(|app| {
            let storage = Storage::new(app.handle())?;
            app.manage(StorageState(Mutex::new(storage)));
            match config::parse(app.handle()) {
                Ok(_) => println!("Config parsed successfully"),
                Err(e) => eprintln!("Failed to parse config: {}", e),
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            entity::event::add_event,
            entity::event::event_content,
            entity::event::write_content,
            entity::event::put_event,
            entity::event::delete_event,
            entity::list::new_list,
            entity::list::get_lists,
            entity::list::delete_list,
            entity::list::list_content,
            entity::tag::add_tag,
            entity::tag::get_tags,
            entity::tag::delete_tag,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    Ok(())
}
