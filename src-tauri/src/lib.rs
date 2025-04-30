use std::ops::DerefMut;

mod entity; // 核心数据实体和存储定义
mod utils; // 通用工具函数

mod debug; // 调试工具
mod function; // 功能

use entity::{event, list, tag, Repository};
use entity::{Storage, StorageState};
use std::sync::Mutex;
use tauri::Manager;
use utils::AppPaths;

pub use entity::{Event, List, Tag};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() -> std::io::Result<()> {
    tauri::Builder::default()
        .setup(|app| {
            AppPaths::init(app.handle())?;
            let app_instance = entity::App::new(app.handle());
            let storage = Storage::new()?;
            app.manage(StorageState(Mutex::new(storage), Mutex::new(app_instance)));
            init_table(&app.state::<StorageState>());
            match utils::config::parse() {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("Error parsing config: {}", e);
                }
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

fn init_table(state: &StorageState) -> () {
    let mut guard = state.0.lock().unwrap();
    let storage = guard.deref_mut();
    Repository::<Event>::ensure_table_exists(storage).ok();
    Repository::<List>::ensure_table_exists(storage).ok();
    Repository::<Tag>::ensure_table_exists(storage).ok();
    ()
}
