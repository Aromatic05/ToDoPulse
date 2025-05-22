use anyhow::Result;
use notify_rust::Notification;

use crate::utils::config::{info, Info};
use crate::utils::manager::scheduled_task::ScheduledTask;
use crate::utils::manager::tasker::TaskManager;

pub fn notify_desktop(title: &str, content: &str) -> Result<()> {
    let mut notification = Notification::new();
    notification.summary(title).body(content);
    notification.show()?;
    Ok(())
}

fn create_notify_desktop_task(
    title: &str,
    content: &str,
    time: Vec<&str>,
) -> Result<ScheduledTask<impl Fn() + Send + Sync + 'static>> {
    let title = title.to_string();
    let content = content.to_string();
    let task = ScheduledTask::new("notify_desktop_task", time, move || {
        notify_desktop(&title, &content).unwrap();
    })?;
    Ok(task)
}

pub async fn setup() {
    let info_config = info().unwrap_or_else(|_| {
        log::error!("Info config not found");
        Info {
            switch: false,
            time: None,
        }
    });
    let time = info_config.time.unwrap_or_default();
    if info_config.switch {
        let task =
            create_notify_desktop_task("", "content", time.iter().map(|s| s.as_str()).collect())
                .unwrap();
        TaskManager::start(Box::new(task)).await;
    }
}
