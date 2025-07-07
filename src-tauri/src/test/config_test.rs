#[cfg(test)]
mod tests {
    use crate::utils::config::*;
    use std::path::PathBuf;
    use tempfile::tempdir;
    use std::fs;

    const CONFIG_FILE: &str = "config.toml";
    
    // 创建测试环境
    fn setup_test_env() -> PathBuf {
        // 创建临时目录作为配置目录
        let temp_dir = tempdir().unwrap();
        let config_dir = temp_dir.path().to_path_buf();
        config_dir
    }

    #[test]
    fn test_custom_config_values() {
        let config_dir = setup_test_env();

        // 写入自定义配置
        let config_content = r#"
        [theme]
        color = "red"
        [info]
        switch = false
        time = ["0 10 * * *"]
        [model]
        switch = true
        name = "test-model"
        tokens = "1024"
        api = "https://api.test.com/v1/chat/completions"
        [webdav]
        enabled = true
        host = "https://webdav-1690957.pd1.123pan.cn/webdav/webdav"
        username = "username"
        password = "passwd"
        remote_dir = "/ToDoPulse"
        "#;

        fs::create_dir_all(&config_dir).unwrap();
        fs::write(config_dir.join(CONFIG_FILE), config_content).unwrap();

        // 使用自定义路径解析配置
        parse_with_path(Some(&config_dir)).unwrap();
        let webdav_config = WebDav::load().unwrap();

        assert_eq!(webdav_config.enabled, true);
        assert_eq!(
            webdav_config.host,
            "https://webdav-1690957.pd1.123pan.cn/webdav/webdav"
        );
    }
}
