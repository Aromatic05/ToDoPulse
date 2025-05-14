use anyhow::{anyhow, Result};
use std::sync::{Arc, OnceLock};
use tokio::sync::{oneshot, Mutex};
use tokio::task::JoinHandle;
use tokio::time::Duration;

static TASK_MANAGER: OnceLock<Arc<Mutex<Vec<TaskWrapper>>>> = OnceLock::new();

pub trait Tasker: Send + Sync {
    fn start(&self) -> &JoinHandle<()>;
    fn stop(&self) -> bool;
    fn reset(&self);
    fn status(&self) -> &JoinHandle<()>;
    fn start_monitor(&self) -> JoinHandle<()> {
        start_monitor()
    }
}

// 任务包装器，添加循环控制功能
struct TaskWrapper {
    task: Box<dyn Tasker>,
    task_monitor: Option<JoinHandle<()>>, // 监控任务状态的句柄
}

/// 获取任务管理器的全局实例
pub fn get_task_manager() -> Arc<Mutex<Vec<TaskWrapper>>> {
    TASK_MANAGER
        .get_or_init(|| Arc::new(Mutex::new(Vec::new())))
        .clone()
}

/// 任务管理器提供的功能
pub struct TaskManager;

impl TaskManager {
    /// 启动指定任务
    pub async fn start(task: Box<dyn Tasker>) -> Result<()> {
        let manager = get_task_manager();
        let mut tasks = manager.lock().await;
        let wrapper = TaskWrapper {
            task,
            task_monitor: None,
        };
        tasks.push(wrapper);
        let index = tasks.len() - 1;

        tasks[index].task.start();

        if tasks[index].task_monitor.is_none() {
            // 释放锁后启动监控
            drop(tasks);
        }

        Ok(())
    }

    /// 停止指定任务
    pub async fn stop(index: usize) -> Result<()> {
        let manager = get_task_manager();
        let mut tasks = manager.lock().await;

        if index >= tasks.len() {
            return Err(anyhow!("任务索引 {} 超出范围", index));
        }

        tasks[index].task.stop();

        if let Some(handle) = tasks[index].task_monitor.take() {
            handle.abort();
        }
        tasks.remove(index);
        Ok(())
    }

    /// 重置指定任务
    pub async fn reset(index: usize) -> Result<()> {
        let manager = get_task_manager();
        let mut tasks = manager.lock().await;

        if index >= tasks.len() {
            return Err(anyhow!("任务索引 {} 超出范围", index));
        }

        tasks[index].task.reset();

        // 停止任务监控
        if let Some(handle) = tasks[index].task_monitor.take() {
            handle.abort();
        }

        Ok(())
    }

    /// 停止所有任务
    pub async fn stop_all() {
        let manager = get_task_manager();
        let mut tasks = manager.lock().await;

        for wrapper in tasks.iter_mut() {
            wrapper.task.stop();

            // 停止任务监控
            if let Some(handle) = wrapper.task_monitor.take() {
                handle.abort();
            }
        }
        tasks.clear();
    }

    /// 重置所有任务
    pub async fn reset_all() {
        let manager = get_task_manager();
        let mut tasks = manager.lock().await;

        for wrapper in tasks.iter_mut() {
            wrapper.task.reset();

            // 停止任务监控
            if let Some(handle) = wrapper.task_monitor.take() {
                handle.abort();
            }
        }
    }
}

/// 任务监控默认实现
fn start_monitor() -> JoinHandle<()> {
    tokio::spawn(async move {
        todo!()
    })
}
