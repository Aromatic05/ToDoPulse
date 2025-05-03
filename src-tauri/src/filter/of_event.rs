use chrono::{self,NaiveDateTime, NaiveTime};

use anyhow::{Ok, Result};


use crate::entity::Event;

use super::Filter;

const ONE_DAY: u64 = 24 * 60 * 60 * 1000;
const ONE_WEEK: u64 = 7 * ONE_DAY;

pub fn map_filter(filter: &str)->Result<Filter<Event>> {
    match filter {
        "today" => Ok(|event| today_filter(event)),
        "tomorrow" => Ok(|event| tomorrow_filter(event)),
        "next_week" => Ok(|event| next_week_filter(event)),
        _ =>  Err(anyhow::anyhow!("Invalid filter")),
    }
} 

// 辅助函数: 为给定日期偏移和时间范围创建筛选函数
fn time_range_filter(entity: &Event, days_offset: i64, range_duration: u64) -> bool {
  let now = chrono::Local::now();
  let target_date = now.date_naive() + chrono::Duration::days(days_offset);
  let time = NaiveTime::from_hms_opt(0, 0, 0).unwrap();
  let date_time = NaiveDateTime::new(target_date, time);
  let timestamp = date_time.and_utc().timestamp() as u64;
  
  entity.task_time.unwrap_or(0) >= timestamp
      && entity.task_time.unwrap_or(0) < timestamp + range_duration
}

fn today_filter(entity: &Event) -> bool {
  time_range_filter(entity, 0, ONE_DAY)
}

fn tomorrow_filter(entity: &Event) -> bool {
  time_range_filter(entity, 1, ONE_DAY)
}

fn next_week_filter(entity: &Event) -> bool {
  time_range_filter(entity, 7, ONE_WEEK)
}