mod desktop;
mod stamp;

use std::sync::mpsc;
use std::time::Duration;
use std::str::FromStr;
use std::thread::JoinHandle;
use cron::Schedule;

use crate::utils::config;
use self::desktop::notify_desktop;

pub struct Task {
    name: String,
    func: Box<dyn Fn() + Send>,
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
    fn start(self) -> JoinHandle<()> {
        let (_, rx) = mpsc::channel::<()>();
        let schedule = self.schedule.clone();

        std::thread::spawn(move || {
            loop {
                if let Ok(_) = rx.try_recv() {
                    break;
                }
                for s in &schedule {
                    if let Some(next) = s.upcoming(chrono::Utc).next() {
                        let now = chrono::Utc::now();
                        if next <= now {
                            (self.task.func)();
                            break;
                        }
                    }
                }
                std::thread::sleep(Duration::from_secs(60));
            }
        })
    }
    #[allow(dead_code)]
    fn stop(handle: JoinHandle<()>) {
        if let Err(e) = handle.join() {
            log::error!("Failed to stop task: {:?}", e);
        }
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
        func: Box::new(|| {
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
    let task_name = task.name.clone();
    if get_schedule(&task_name).is_empty() {
        log::error!("No schedule found for task: {}", task_name);
        return std::thread::spawn(|| {});
    }
    let schedule_manager = ScheduleManager::new(task);
    log::info!("Starting task: {}", task_name);
    schedule_manager.start()
}