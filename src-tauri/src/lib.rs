mod ipc;
mod storage;
mod aigc;
mod config;
mod filter;
mod data;

use std::sync::Mutex;
use storage::{Storage, StorageState};
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() -> std::io::Result<()> {
    tauri::Builder::default()
        .setup(|app|{
          let storage = Storage::new(app.handle())?;
          app.manage(StorageState(Mutex::new(storage)));
          Ok(())
        }
        )
        .invoke_handler(tauri::generate_handler![
            ipc::new_event,
            ipc::add_event,
            ipc::delete_event,
            ipc::get_events,
            ipc::get_metadata,
            ipc::new_list,
            ipc::add_tag,
            config::parse,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    Ok(())
}
