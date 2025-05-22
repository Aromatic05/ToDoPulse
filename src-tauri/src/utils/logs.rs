use env_logger::Builder;
use std::fs::OpenOptions;
use std::io::Write;

use crate::utils::AppPaths;

const LOG_FILE: &str = "app.log";

pub fn init_log() {
    let log_path = AppPaths::log_dir();
    if !log_path.exists() {
        std::fs::create_dir_all(&log_path).unwrap();
    }
    let log_file = log_path.join(LOG_FILE);
    let log = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(log_file)
        .expect("Failed to open log file");
    Builder::new()
        .filter_level(log::LevelFilter::Info)
        .format(|buf, record| {
            writeln!(
                buf,
                "{} - {}: {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                record.args()
            )
        })
        .target(env_logger::Target::Pipe(Box::new(log)))
        .init();
    log::info!("Log initialized at {}", log_path.display());
}
