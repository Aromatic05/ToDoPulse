use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;
use tauri_plugin_dialog::DialogExt;

use crate::utils::AppPaths;

/// Gets the path to the export directory
///
/// Returns the absolute path to the application's export directory
/// where exported files are saved by default.
///
/// # Returns
/// * `Result<String, String>` - Path to the export directory or error message
#[tauri::command]
pub async fn get_export_directory() -> Result<String, String> {
    Ok(AppPaths::export_dir()
        .to_str()
        .ok_or("无法转换导出目录路径为字符串")?
        .to_string())
}

/// Opens a file save dialog for the user to choose where to save the export
///
/// Opens a native file save dialog with appropriate filters based on the file type.
/// The dialog will suggest a default filename with the appropriate extension.
///
/// # Parameters
/// * `app_handle` - Handle to the Tauri application
/// * `suggested_name` - Suggested filename (without extension)
/// * `extension` - File extension for the export format (e.g., "ics", "json", "md")
///
/// # Returns
/// * `Result<Option<String>, String>` - Selected file path or None if cancelled
#[tauri::command]
pub async fn select_save_path(
    app_handle: tauri::AppHandle,
    suggested_name: String,
    extension: String,
) -> Result<Option<String>, String> {
    // 构建默认文件名
    let default_filename = format!("{}.{}", suggested_name, extension);

    // 设置文件过滤器
    let filter_name = match extension.to_lowercase().as_str() {
        "ics" => "Calendar Files",
        "json" => "JSON Files",
        "md" => "Markdown Files",
        "csv" => "CSV Files",
        _ => "Text Files",
    };

    // 使用新的对话框 API
    let file_path = app_handle
        .dialog()
        .file()
        .add_filter(filter_name, &[&extension])
        .set_file_name(&default_filename)
        .blocking_save_file();

    match file_path {
        Some(path) => Ok(Some(path.to_string())),
        None => Ok(None), // 用户取消了选择
    }
}

/// Saves exported content to a file
///
/// Writes the provided content to a file, either at a custom path specified by the user
/// or in the default export directory. Ensures the path is valid and directories exist.
///
/// # Parameters
/// * `content` - The content to write to the file
/// * `filename` - Base name for the file (without extension)
/// * `format` - Export format, used to determine the file extension
/// * `custom_path` - Optional custom path where the file should be saved
///
/// # Returns
/// * `Result<String, String>` - Path to the saved file or error message
#[tauri::command]
pub async fn save_export_file(
    content: String,
    filename: String,
    format: String,
    custom_path: Option<String>,
) -> Result<String, String> {
    // 确定保存路径
    let file_path = if let Some(path) = custom_path {
        // 使用用户提供的路径
        let path = Path::new(&path);
        if path.is_dir() {
            // 如果是目录，则在该目录下创建文件
            let extension = match format.to_lowercase().as_str() {
                "ics" => "ics",
                "json" => "json",
                "markdown" | "md" => "md",
                "csv" => "csv",
                _ => "txt",
            };

            let safe_filename = sanitize_filename(&filename);
            let full_filename = format!("{}.{}", safe_filename, extension);
            PathBuf::from(path).join(full_filename)
        } else {
            // 如果已经是文件路径，则直接使用
            PathBuf::from(path)
        }
    } else {
        // 使用默认的导出目录
        let export_dir = match get_export_directory().await {
            Ok(dir) => PathBuf::from(dir),
            Err(e) => return Err(format!("获取导出目录失败: {}", e)),
        };

        // 根据格式创建扩展名
        let extension = match format.to_lowercase().as_str() {
            "ics" => "ics",
            "json" => "json",
            "markdown" | "md" => "md",
            "csv" => "csv",
            _ => "txt",
        };

        // 创建文件名
        let safe_filename = sanitize_filename(&filename);
        let full_filename = format!("{}.{}", safe_filename, extension);
        export_dir.join(full_filename)
    };

    // 创建必要的目录
    if let Some(parent) = file_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).map_err(|e| format!("无法创建目录: {}", e))?;
        }
    }

    // 保存文件
    fs::write(&file_path, content).map_err(|e| format!("保存文件失败: {}", e))?;

    Ok(file_path.to_string_lossy().to_string())
}

/// 清理文件名，移除不安全的字符
fn sanitize_filename(filename: &str) -> String {
    // 移除不安全字符
    let forbidden_chars = r#"/\?%*:|"<>#;={}@^~[]`"#;
    let mut safe_filename = filename.to_string();
    for c in forbidden_chars.chars() {
        safe_filename = safe_filename.replace(c, "_");
    }

    // 移除前导和尾部空格
    safe_filename = safe_filename.trim().to_string();

    // 如果文件名为空，则使用默认名
    if safe_filename.is_empty() {
        safe_filename = "export".to_string();
    }

    safe_filename
}
