use once_cell::sync::Lazy;
use parking_lot::Mutex;
use std::path::{Path, PathBuf};

pub trait WebDavConfig: Send + Sync {
    fn enabled(&self) -> bool;
    fn remote_dir(&self) -> &Path;
    fn credential(&self) -> (&str, &str, &str);
}

pub trait AppPath: Send + Sync {
    fn data_dir(&self) -> PathBuf;
    fn config_dir(&self) -> PathBuf;
}

macro_rules! define_global_manager {
    ($name:ident, $trait:path, $error_msg:literal) => {
        static $name: Lazy<Mutex<Option<Box<dyn $trait>>>> = Lazy::new(|| Mutex::new(None));

        ::paste::paste! {
            #[allow(dead_code)]
            pub fn [<init_ $name:lower>](provider: impl $trait + 'static) {
                let mut guard = $name.lock();
                *guard = Some(Box::new(provider));
            }
        }

        ::paste::paste! {
            pub fn [<with_ $name:lower>]<F, R>(f: F) -> R
            where
                F: FnOnce(&dyn $trait) -> R,
            {
                let guard = $name.lock();
                let provider = guard
                    .as_ref()
                    .expect($error_msg)
                    .as_ref();
                f(provider)
            }
        }
    };
}

// --- 使用宏来生成具体的实现 ---
define_global_manager!(CONFIG, WebDavConfig, "WebDAV配置未加载");
define_global_manager!(APP_PATH, AppPath, "应用路径未加载");

#[macro_export]
macro_rules! with_config_if_enabled {
    (|$config_ident:ident| $if_body:expr, else $else_body:expr) => {
        with_config(|$config_ident| {
            if $config_ident.enabled() {$if_body} else {$else_body}
        })
    };
}

#[macro_export]
macro_rules! without_config_enabled {
    ($body:expr) => {
        if !with_config(|config| config.enabled()) {
            $body;
        }
    };
}
