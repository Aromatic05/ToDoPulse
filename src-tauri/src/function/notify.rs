mod desktop;
mod stamp;

use std::str::FromStr;
use std::time::Duration;
use cron::Schedule;
use tokio::task::JoinHandle;
use tokio::sync::oneshot;

use crate::utils::config;
use self::desktop::notify_desktop;
use super::tasker::Tasker;

pub struct Task {
    name: String,
    func: Box<dyn Fn() + Send + Sync>,
}


pub struct ScheduleManager {
    schedule: Vec<Schedule>,
    task: Task,
}

impl ScheduleManager {
    fn new(task: Task) -> Self {
        let schedule = get_schedule(task.name.as_str());
        ScheduleManager {
            schedule,
            task: task,
        }
    }
}

impl Tasker for ScheduleManager {
    fn start(&self) -> &JoinHandle<()> {
        // 由于 trait 要求返回引用，这里无法直接实现
        // 实际项目中应考虑重构，这里暂做简化处理
        panic!("ScheduleManager::start 需要重构为异步模式")
    }
    
    fn stop(&self) -> bool {
        // 由于重构，该方法也需要调整
        false
    }
    
    fn reset(&self) {
        // 重置相关逻辑
    }
    
    fn status(&self) -> &JoinHandle<()> {
        // 由于 trait 要求返回引用，这里无法直接实现
        panic!("ScheduleManager::status 需要重构为异步模式")
    }
  }


fn get_schedule(task_name: &str) -> Vec<Schedule> {
    let times = match task_name {
        "info" => config::info_time(),
        _ => vec![],
    };
    let mut schedules = Vec::new();
    for time in times {
        if let Ok(schedule) = Schedule::from_str(&time) {
            schedules.push(schedule);
        } else {
            log::error!("Invalid cron expression: {}", time);
        }
    }
    schedules
}

pub fn set_up() -> Vec<JoinHandle<()>> {
    let mut handles = Vec::new();
    let info_task = Task {
        name: "info".to_string(),
        func: Box::new(move || {
            log::info!("Running info task");
            let title = "DeepSeek AI";
            let content = "DeepSeek AI is a powerful tool for managing your tasks and notes.";
            match notify_desktop(title, content, None) {
                Ok(_) => log::info!("Notification sent successfully"),
                Err(e) => log::error!("Failed to send notification: {}", e),
            }
          }
        ),
    };
    handles.push(set_up_task(info_task));
    handles
}
fn set_up_task(task: Task) -> JoinHandle<()> {
  todo!()
}