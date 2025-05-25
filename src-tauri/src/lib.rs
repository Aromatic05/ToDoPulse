mod entity; // 核心数据实体和存储定义
mod error; // 错误处理
mod filter;
mod function; // 功能
mod init; // 初始化模块
mod utils; // 通用工具函数

use entity::{event, list, tag};
use function::{export, sync, upload};
use tauri_plugin_dialog;
use utils::{config, init_tray};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() -> std::io::Result<()> {
    tauri::Builder::default()
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                window.hide().unwrap();
                api.prevent_close();
            }
        })
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            // 初始化系统托盘
            if let Err(e) = init_tray(&app.handle()) {
                log::error!("Error initializing tray: {}", e);
            }
            
            tauri::async_runtime::block_on(async {
                let res = init::initialize_app(app).await;
                if let Err(e) = res {
                    log::error!("Error initializing app: {}", e);
                }
            });
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
            tag::tag_content,
            export::export_events,
            export::export_list_events,
            export::export_all_events,
            export::export_events_by_status,
            export::export_events_by_date_range,
            export::save::get_export_directory,
            export::save::save_export_file,
            export::save::select_save_path,
            sync::test_webdav_connection,
            sync::sync_now,
            sync::get_sync_status,
            config::update_config,
            upload::upload_file,
            upload::save_remote_image,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    Ok(())
}
