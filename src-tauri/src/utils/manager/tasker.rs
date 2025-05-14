use anyhow::{anyhow, Result};
use async_trait::async_trait;
use std::sync::{Arc, OnceLock};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

static TASK_MANAGER: OnceLock<Arc<Mutex<Vec<TaskWrapper>>>> = OnceLock::new();
static TASK_MAP: OnceLock<Arc<Mutex<Vec<String>>>> = OnceLock::new();

#[async_trait]
pub trait Tasker: Send + Sync {
    fn start(&self) -> JoinHandle<()>;
    fn name(&self) -> String;
    async fn stop(&self) -> bool;
}

struct TaskWrapper {
    task: Box<dyn Tasker>,
    task_monitor: Option<JoinHandle<()>>, // 监控任务状态的句柄
}

/// 获取任务管理器的全局实例
fn get_task_manager() -> Arc<Mutex<Vec<TaskWrapper>>> {
    TASK_MANAGER
        .get_or_init(|| Arc::new(Mutex::new(Vec::new())))
        .clone()
}

fn get_task_map() -> Arc<Mutex<Vec<String>>> {
    TASK_MAP
        .get_or_init(|| Arc::new(Mutex::new(Vec::new())))
        .clone()
}

/// 任务管理器提供的功能
pub struct TaskManager;

impl TaskManager {
    /// 启动指定任务
    pub async fn start(task: Box<dyn Tasker>) {
        let name = task.name();
        let task_map = get_task_map();
        let mut task_map = task_map.lock().await;
        task_map.push(format!("任务 {}", name));
        log::info!("任务管理器：任务 {} 启动成功", name);
        let handle = task.start();
        let manager = get_task_manager();
        let mut tasks = manager.lock().await;
        let wrapper = TaskWrapper {
            task,
            task_monitor: Some(handle),
        };
        tasks.push(wrapper);
    }

    #[allow(dead_code)]
    pub async fn stop(name: &str) -> Result<()> {
        let task_map = get_task_map();
        let task_map = task_map.lock().await;
        let index = task_map
            .iter()
            .position(|task_name| task_name == name)
            .ok_or_else(|| anyhow!("未找到任务 {}", name))?;
        let manager = get_task_manager();
        let mut tasks = manager.lock().await;
        if index >= tasks.len() {
            return Err(anyhow!("任务索引 {} 超出范围", index));
        }
        tasks[index].task.stop().await;
        if let Some(handle) = tasks[index].task_monitor.take() {
            handle.abort();
        }
        tasks.remove(index);
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn stop_all() {
        let manager = get_task_manager();
        let mut tasks = manager.lock().await;
        for task in tasks.iter_mut() {
            if let Some(handle) = task.task_monitor.take() {
                handle.abort();
            }
            task.task.stop().await;
        }
        tasks.clear();
        let task_map = get_task_map();
        let mut task_map = task_map.lock().await;
        task_map.clear();
        log::info!("任务管理器：所有任务已停止");
    }
}

pub fn init_task_manager() {
    let _ = TASK_MANAGER.get_or_init(|| Arc::new(Mutex::new(Vec::new())));
    let _ = TASK_MAP.get_or_init(|| Arc::new(Mutex::new(Vec::new())));
}
