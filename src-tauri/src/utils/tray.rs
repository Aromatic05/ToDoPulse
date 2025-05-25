#[cfg(desktop)]
use tauri::{
    menu::{Menu, MenuItem},
    tray::{TrayIconBuilder},
    AppHandle, Emitter, Manager,
};

/// 初始化系统托盘
#[cfg(desktop)]
pub fn init_tray(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    // 创建托盘菜单项
    let settings_i = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
    let toggle_i = MenuItem::with_id(app, "toggle", "Toggle Window", true, None::<&str>)?;
    let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&settings_i, &toggle_i, &quit_i])?;

    let _tray = TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .tooltip("ToDoPulse - Task Manager")
        .on_menu_event(move |app, event| match event.id.as_ref() {
            "settings" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                    let _ = app.emit_to("main", "toggle-settings", ());
                }
            }
            "toggle" => {
                if let Some(window) = app.get_webview_window("main") {
                    let visible = window.is_visible().unwrap_or(false);
                    if visible {
                        let _ = window.hide();
                    } else {
                        let _ = window.show();
                        let _ = window.unminimize();
                    }
                }
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .build(app)?;

    Ok(())
}
