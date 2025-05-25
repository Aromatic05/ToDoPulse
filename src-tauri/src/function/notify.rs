use anyhow::Result;
use notify_rust::Notification;

use crate::utils::config::Info;
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
        notify_desktop(&title, &content)
        .unwrap_or_else(|e| log::error!("Failed to send notification: {}", e));
    })?;
    Ok(task)
}

pub async fn setup() {
    let info_config = Info::load().unwrap_or_else(|_| {
        log::error!("Info config not found");
        Info {
            switch: false,
            time: None,
        }
    });
    
    // 如果开关关闭或没有时间配置，直接返回
    if !info_config.switch || info_config.time.is_none() {
        return;
    }
    
    let time = info_config.time.unwrap();
    
    // 尝试创建任务，失败就返回
    let task = match create_notify_desktop_task("", "content", time.iter().map(|s| s.as_str()).collect()) {
        Ok(task) => task,
        Err(e) => {
            log::error!("Failed to create notification task: {}", e);
            return;
        }
    };
    
    // 启动成功创建的任务
    TaskManager::start(Box::new(task)).await;
}
