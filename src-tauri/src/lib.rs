mod ipc;
mod storage;
mod aigc;
mod config;
mod data;
mod debug;
mod time;
mod utils;

use std::sync::Mutex;
use storage::{Storage, StorageState};
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() -> std::io::Result<()> {
    tauri::Builder::default()
        .setup(|app|{
          let storage = Storage::new(app.handle())?;
          app.manage(StorageState(Mutex::new(storage)));
          match config::parse(app.handle()) {
            Ok(_) => println!("Config parsed successfully"),
            Err(e) => eprintln!("Failed to parse config: {}", e),
          }
          Ok(())
        }
        )
        .invoke_handler(tauri::generate_handler![
            ipc::add_event,
            ipc::event_content,
            ipc::write_content,
            ipc::put_event,
            ipc::delete_event,
            ipc::new_list,
            ipc::get_lists,
            ipc::delete_list,
            ipc::list_content,
            ipc::add_tag,
            ipc::get_tags,
            ipc::delete_tag,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    Ok(())
}
