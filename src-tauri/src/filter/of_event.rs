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
  let timestamp = date_time.and_utc().timestamp() as u64*1000;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::Event;
    use chrono::Duration;

    fn create_event_with_time(days_from_now: i64, hours: u32) -> Event {
        let mut event = Event::new("Test Event", "This is a test event.");
        let now = chrono::Local::now().naive_local();
        let target_date = now.date() + Duration::days(days_from_now);
        let time = chrono::NaiveTime::from_hms_opt(hours, 0, 0).unwrap();
        let date_time = chrono::NaiveDateTime::new(target_date, time);
        
        // Convert to timestamp (seconds since epoch)
        event.task_time = Some(date_time.and_utc().timestamp() as u64);
        event.task_time = Some(event.task_time.unwrap() * 1000); // Convert to milliseconds
        event
    }

    #[test]
    fn test_today_filter() {
        // Event with today's date at 10:00
        let today_event = create_event_with_time(0, 0);
        println!("Event time: {:?}", today_event.task_time);
        assert!(today_filter(&today_event), "Should match event scheduled for today");
        
        // Event with yesterday's date
        let yesterday_event = create_event_with_time(-1, 0);
        println!("Event time: {:?}", yesterday_event.task_time);
        assert!(!today_filter(&yesterday_event), "Should not match event scheduled for yesterday");
        
        // Event with tomorrow's date
        let tomorrow_event = create_event_with_time(1, 0);
        println!("Event time: {:?}", tomorrow_event.task_time);
        assert!(!today_filter(&tomorrow_event), "Should not match event scheduled for tomorrow");
    }

    #[test]
    fn test_tomorrow_filter() {
        // Event with tomorrow's date
        let tomorrow_event = create_event_with_time(1, 0);
        assert!(tomorrow_filter(&tomorrow_event), "Should match event scheduled for tomorrow");
        
        // Event with today's date
        let today_event = create_event_with_time(0, 0);
        assert!(!tomorrow_filter(&today_event), "Should not match event scheduled for today");
        
        // Event with day after tomorrow
        let day_after_event = create_event_with_time(2, 0);
        assert!(!tomorrow_filter(&day_after_event), "Should not match event scheduled for day after tomorrow");
    }

    #[test]
    fn test_next_week_filter() {
        // Event exactly one week from now
        let next_week_event = create_event_with_time(7, 0);
        assert!(next_week_filter(&next_week_event), "Should match event scheduled for exactly one week from now");
        
        // Event 10 days from now (within next week range)
        let within_week_event = create_event_with_time(10, 0);
        assert!(next_week_filter(&within_week_event), "Should match event scheduled within next week range");
        
        // Event 15 days from now (outside next week range)
        let outside_week_event = create_event_with_time(15, 0);
        assert!(!next_week_filter(&outside_week_event), "Should not match event scheduled outside next week range");
        
        // Today's event
        let today_event = create_event_with_time(0, 0);
        assert!(!next_week_filter(&today_event), "Should not match event scheduled for today");
    }
}