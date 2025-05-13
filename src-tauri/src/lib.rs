mod debug; // 调试工具
mod entity; // 核心数据实体和存储定义
mod error; // 错误处理
mod filter;
mod function; // 功能
mod utils; // 通用工具函数

use entity::{event, list, tag};
use entity::{Storage, StorageState};
use tauri::Manager;
use tokio::sync::Mutex;
use utils::logs::init_log;
use utils::AppPaths;

use function::export;
use function::sync;
use tauri_plugin_dialog;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() -> std::io::Result<()> {
    function::notify::set_up();
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            AppPaths::init(app.handle())?;
            init_log();
            let app_instance = entity::App::new(app.handle());
            let storage = Storage::new()?;
            app.manage(StorageState(Mutex::new(storage), Mutex::new(app_instance)));
            
            // 注册同步功能命令
            sync::register_sync_commands(app)?;
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
            event::update_event,
            event::delete_event,
            event::filter_events,
            list::new_list,
            list::get_lists,
            list::delete_list,
            list::rename_list,
            list::list_content,
            tag::add_tag,
            tag::get_tags,
            tag::delete_tag,
            export::export_events,
            export::export_list_events,
            export::export_all_events,
            export::export_events_by_status,
            export::export_events_by_date_range,
            export::save::get_export_directory,
            export::save::save_export_file,
            export::save::select_save_path,
            // WebDAV同步命令
            sync::test_webdav_connection,
            sync::sync_now,
            sync::get_sync_status,
            sync::update_sync_config,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    Ok(())
}
