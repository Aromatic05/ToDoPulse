use std::path::{Path, PathBuf};
use log::{debug, error, warn};

pub(crate) struct PathResolver {}

impl PathResolver {
    /// 从完整的WebDAV路径提取标准化路径
    /// 输入: "/webdav/webdav/ToDoPulse/5555.md"
    /// 输出: "/ToDoPulse/5555.md"
    pub(crate) fn extract_relative_path(&self, webdav_url: &Path) -> Option<PathBuf> {
        let webdav_url = webdav_url.as_os_str().to_string_lossy();
        debug!("=== extract_relative_path 开始 ===");
        debug!("输入路径: '{}'", webdav_url);

        // URL解码
        let decoded = match urlencoding::decode(&webdav_url) {
            Ok(d) => d.into_owned(),
            Err(e) => {
                warn!("URL解码失败: {} - {}", webdav_url, e);
                return None;
            }
        };
        debug!("URL解码后: '{}'", decoded);

        // 固定移除 "/webdav/webdav" 前缀
        const WEBDAV_PREFIX: &str = "/webdav/webdav";

        if decoded.starts_with(WEBDAV_PREFIX) {
            let after_prefix = &decoded[WEBDAV_PREFIX.len()..];

            // 如果结果为空或只是斜杠，表示这是根目录
            if after_prefix.is_empty() || after_prefix == "/" {
                debug!("检测到根目录，返回空字符串");
                return Some(PathBuf::new());
            }

            // 确保以斜杠开头的标准格式
            let normalized = format!("/{}", after_prefix.trim_start_matches('/'));

            debug!("最终标准化结果: '{}'", normalized);
            Some(PathBuf::from(normalized))
        } else {
            warn!("路径不以webdav前缀开头: '{}'", decoded);
            None
        }
    }
}
