use super::tasker::Tasker;
use async_trait::async_trait;
use chrono::{Local, NaiveTime, Timelike};
use log::info;
use std::sync::Arc;
use tokio::pin;
use tokio::sync::{oneshot, Mutex};
use tokio::task::JoinHandle;
use tokio::time::{sleep, Duration};

const CHECK_INTERVAL: Duration = Duration::from_secs(60);

pub struct ScheduledTask<F>
where
    F: Fn() + Send + Sync + 'static,
{
    name: String,
    // 要执行的时间点列表，格式为 "HH:MM:SS"
    exec_times: Vec<NaiveTime>,
    func: Arc<F>,
    stop_signal: Arc<Mutex<Option<oneshot::Sender<()>>>>,
}

impl<F> ScheduledTask<F>
where
    F: Fn() + Send + Sync + 'static,
{
    /// 创建一个按特定时间点执行的任务
    ///
    /// # 参数
    ///
    /// * `name` - 任务名称
    /// * `times` - 时间点列表，每个时间格式为 "HH:MM:SS" 或 "HH:MM"
    /// * `function` - 要执行的函数
    pub fn new(
        name: impl Into<String>,
        times: Vec<&str>,
        function: F,
    ) -> Result<Self, chrono::ParseError> {
        let mut exec_times = Vec::with_capacity(times.len());

        for time_str in times {
            // 尝试解析时间
            let format = if time_str.len() == 5 {
                "%H:%M"
            } else {
                "%H:%M:%S"
            };
            let time = NaiveTime::parse_from_str(time_str, format)?;
            exec_times.push(time);
        }

        Ok(ScheduledTask {
            name: name.into(),
            exec_times,
            func: Arc::new(function),
            stop_signal: Arc::new(Mutex::new(None)),
        })
    }
}

#[async_trait]
impl<F> Tasker for ScheduledTask<F>
where
    F: Fn() + Send + Sync + 'static,
{
    fn start(&self) -> JoinHandle<()> {
        let (stop_tx, stop_rx) = oneshot::channel();
        let stop_signal = self.stop_signal.clone();

        tokio::spawn(async move {
            let mut sender = stop_signal.lock().await;
            *sender = Some(stop_tx);
        });

        let name = self.name.clone();
        let times = self.exec_times.clone();
        let f = self.func.clone();

        let handle = tokio::spawn(async move {
            info!("Started scheduled task: {}", name);

            pin!(stop_rx);
            // 持续检查时间
            loop {
                tokio::select! {
                  _ = & mut stop_rx => {
                      info!("Stopping scheduled task: {}", name);
                      break;
                  }
                  _ = sleep(CHECK_INTERVAL) => {
                      let now = Local::now().time();
                      for time in &times {
                          if now.hour() == time.hour() && now.minute() == time.minute() {
                              f();
                              info!("Executed scheduled task: {}", name);
                          }
                      }
                  }
                }
            }
        });
        handle
    }

    fn name(&self) -> String {
        self.name.clone()
    }

    async fn stop(&self) -> bool {
        let mut stop_signal = self.stop_signal.lock().await;
        if let Some(sender) = stop_signal.take() {
            if sender.send(()).is_ok() {
                info!("Scheduled task {} stopped successfully", self.name);
                return true;
            }
        }
        info!("Failed to stop scheduled task {}", self.name);
        false
    }
}
