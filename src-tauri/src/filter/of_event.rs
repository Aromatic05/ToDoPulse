use core::time;

use chrono::{self,NaiveDateTime, NaiveTime};

use anyhow::{Ok, Result};


use crate::entity::Event;

use super::Filter;

pub fn map_filter(filter: &str)->Result<Filter<Event>> {
    match filter {
        "today" => Ok(|event| today_filter(event)),
        _ =>  Err(anyhow::anyhow!("Invalid filter")),
    }
} 

fn time_filter(entity: &Event, time: u64) -> bool{
    return entity.task_time.unwrap_or(0) >= time;
} 

fn today_filter(entity: &Event) -> bool{
    let now = chrono::Local::now();
    let today = now.date_naive();
    let time = NaiveTime::from_hms_opt(0, 0, 0).unwrap();
    let date_time = NaiveDateTime::new(today, time);
    let timestamp = date_time.and_utc().timestamp() as u64;
    time_filter(entity, timestamp)
}