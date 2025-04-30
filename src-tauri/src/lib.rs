mod entity; // 核心数据实体和存储定义
mod time; // 时间处理工具

mod aigc; // AI 生成内容相关功能

mod config; // 配置管理
mod debug; // 调试工具
mod info;
mod utils; // 通用工具函数 // 通知

use entity::{event, list, tag};
use entity::{Storage, StorageState};
use std::sync::Mutex;
use tauri::Manager;

pub use entity::{Event, List, Tag};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() -> std::io::Result<()> {
    tauri::Builder::default()
        .setup(|app| {
            let app_instance = entity::App::new(app.handle());
            let storage = Storage::new(app.handle())?;
            app.manage(StorageState(Mutex::new(storage), Mutex::new(app_instance)));
            match config::parse(app.handle()) {
                Ok(_) => println!("Config parsed successfully"),
                Err(e) => eprintln!("Failed to parse config: {}", e),
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            event::add_event,
            event::event_content,
            event::write_content,
            event::put_event,
            event::delete_event,
            list::new_list,
            list::get_lists,
            list::delete_list,
            list::rename_list,
            list::list_content,
            tag::add_tag,
            tag::get_tags,
            tag::delete_tag,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    Ok(())
}
