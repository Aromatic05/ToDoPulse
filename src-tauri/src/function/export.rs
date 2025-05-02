pub mod ics;
use std::fs;
use std::path::PathBuf;

/// 导出文件的类型
pub enum ExportFormat {
    ICS,
    JSON,
    Markdown,
    CSV,
}

pub struct AppPaths;

impl AppPaths {
    /// Returns the export directory path
    pub fn export_dir() -> PathBuf {
        // Define the export directory path logic here
        let base_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        base_dir.join("exports")
    }
}

/// 获取基础导出目录
#[tauri::command]
pub async fn get_export_directory() -> Result<String, String> {
    let export_dir = AppPaths::export_dir();

    // 确保导出目录存在
    if !export_dir.exists() {
        fs::create_dir_all(&export_dir).map_err(|e| format!("无法创建导出目录: {}", e))?;
    }

    Ok(export_dir.to_string_lossy().to_string())
}

/// 将数据保存到指定文件
#[tauri::command]
pub async fn save_export_file(
    content: String,
    filename: String,
    format: String,
) -> Result<String, String> {
    let export_dir = AppPaths::export_dir();

    // 确保目录存在
    if !export_dir.exists() {
        fs::create_dir_all(&export_dir).map_err(|e| format!("无法创建导出目录: {}", e))?;
    }

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
    let file_path = export_dir.join(full_filename);

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
