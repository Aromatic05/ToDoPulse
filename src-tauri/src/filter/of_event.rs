use chrono::{self, DateTime, NaiveDate};

use anyhow::{Ok, Result};

use super::Filter::{self, A, B};
use crate::entity::Event;
use crate::utils::target_date_timestamp as timestamp;

const ONE_DAY: u64 = 24 * 60 * 60 * 1000;
const ONE_WEEK: u64 = 7 * ONE_DAY;

pub fn map_filter(filter: &str, word_match: Option<bool>) -> Result<Filter<Event>> {
    if let Some(_) = word_match {
        let filter = filter.to_string();
        return Ok(B(Box::new(move |event| {
            // 这里的word_match是一个字符串
            word_match_filter(&event, &filter)
        })));
    }
    match filter {
        "overdue" => Ok(A(|event| overdue_filter(event))),
        "today" => Ok(A(|event| today_filter(event))),
        "tomorrow" => Ok(A(|event| tomorrow_filter(event))),
        "this_week" => Ok(A(|event| this_week_filter(event))),
        "next_week" => Ok(A(|event| next_week_filter(event))),
        date_str => {
            let date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
                .map_err(|_| anyhow::anyhow!("Invalid date filter"))?;
            // log::info!("Parsed date: {}", date);
            Ok(B(Box::new(move |event| time_filter(date, &event))))
        }
    }
}

fn word_match_filter(entity: &Event, word: &str) -> bool {
    let title: &str = entity.title.as_ref();
    let content: &str = entity.content.as_ref();
    title.contains(word) || content.contains(word)
}

// 辅助函数: 为给定日期偏移和时间范围创建筛选函数
fn time_range_filter(entity: &Event, days_offset: i64, range_duration: u64) -> bool {
    let timestamp = timestamp(days_offset);
    entity.task_time.unwrap_or(0) >= timestamp
        && entity.task_time.unwrap_or(0) < timestamp + range_duration
}

fn today_filter(entity: &Event) -> bool {
    time_range_filter(entity, 0, ONE_DAY)
}

fn tomorrow_filter(entity: &Event) -> bool {
    time_range_filter(entity, 1, ONE_DAY)
}

fn this_week_filter(entity: &Event) -> bool {
    // 为了排除今天和明天的事件
    time_range_filter(entity, 2, ONE_WEEK - 2 * ONE_DAY)
}

fn next_week_filter(entity: &Event) -> bool {
    time_range_filter(entity, 7, ONE_WEEK)
}

fn overdue_filter(entity: &Event) -> bool {
    let timestamp = timestamp(0);
    entity.task_time.unwrap_or(0) < timestamp && !entity.finished
}

fn time_filter(day: NaiveDate, event: &Event) -> bool {
    let event_time = event.task_time;
    match event_time {
        Some(time) => time_to_date(&time) == day,
        None => {
            log::error!("Event time is None");
            false
        }
    }
}

fn time_to_date(time: &u64) -> NaiveDate {
    // 毫秒转为秒并保持正确的时区一致性
    let secs = (*time / 1000) as i64;
    let datetime = DateTime::from_timestamp(secs, 0)
        .unwrap_or_else(|| DateTime::from_timestamp(0, 0).unwrap());

    // 转换为本地日期以保持与筛选器一致
    datetime.with_timezone(&chrono::Local).date_naive()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::Event;
    use chrono::{Duration, TimeZone};

    // 正确方法
    fn create_event_with_time(days_from_now: i64, hours: u32) -> Event {
        let mut event = Event::new("Test Event", "This is a test event.");

        // 从本地时间开始
        let now = chrono::Local::now();
        let target_date = now.date_naive() + Duration::days(days_from_now);
        let time = chrono::NaiveTime::from_hms_opt(hours, 0, 0).unwrap();
        let naive_dt = chrono::NaiveDateTime::new(target_date, time);

        // 将naive日期时间正确转换为带时区的日期时间
        let local_dt = chrono::Local.from_local_datetime(&naive_dt).unwrap();

        // 转换为UTC后再获取时间戳
        let utc_dt = local_dt.with_timezone(&chrono::Utc);
        event.task_time = Some((utc_dt.timestamp() as u64) * 1000);

        event
    }

    #[test]
    fn test_today_filter() {
        // Event with today's date at 10:00
        let today_event = create_event_with_time(0, 1);
        println!("Event time: {:?}", today_event.task_time);
        assert!(today_filter(&today_event) == true);

        // Event with yesterday's date
        let yesterday_event = create_event_with_time(-1, 0);
        println!("Event time: {:?}", yesterday_event.task_time);
        assert!(
            !today_filter(&yesterday_event),
            "Should not match event scheduled for yesterday"
        );

        // Event with tomorrow's date
        let tomorrow_event = create_event_with_time(1, 0);
        println!("Event time: {:?}", tomorrow_event.task_time);
        assert!(
            !today_filter(&tomorrow_event),
            "Should not match event scheduled for tomorrow"
        );
    }

    #[test]
    fn test_time_filter() {
        // 获取今天的日期对象
        let today = chrono::Local::now().date_naive();
        println!("Today's date: {}", today);

        // 创建今天的事件
        let today_event = create_event_with_time(0, 10);
        let today_timestamp = today_event.task_time.unwrap();
        println!("Today event time: {:?}", today_timestamp);

        // 验证time_filter能正确匹配今天的事件
        assert!(
            time_filter(today, &today_event),
            "Should match event scheduled for today"
        );

        // 为昨天和明天创建日期对象
        let yesterday = today - Duration::days(1);
        let tomorrow = today + Duration::days(1);

        // 创建昨天和明天的事件
        let yesterday_event = create_event_with_time(-1, 10);
        let tomorrow_event = create_event_with_time(1, 10);

        // 验证匹配正确的日期
        assert!(
            time_filter(yesterday, &yesterday_event),
            "Should match event scheduled for yesterday"
        );
        assert!(
            time_filter(tomorrow, &tomorrow_event),
            "Should match event scheduled for tomorrow"
        );

        // 验证不匹配错误的日期
        assert!(
            !time_filter(today, &yesterday_event),
            "Should not match yesterday's event with today's date"
        );
        assert!(
            !time_filter(today, &tomorrow_event),
            "Should not match tomorrow's event with today's date"
        );
        assert!(
            !time_filter(yesterday, &today_event),
            "Should not match today's event with yesterday's date"
        );
        assert!(
            !time_filter(tomorrow, &today_event),
            "Should not match today's event with tomorrow's date"
        );

        // 测试跨时区边界情况
        // 23:30的事件应该匹配今天而不是明天
        let late_today_event = create_event_with_time(0, 23);
        assert!(
            time_filter(today, &late_today_event),
            "Late today event should match today's date"
        );
        assert!(
            !time_filter(tomorrow, &late_today_event),
            "Late today event should not match tomorrow's date"
        );

        // 00:30的事件应该匹配明天而不是今天
        let early_tomorrow_event = create_event_with_time(1, 0);
        assert!(
            !time_filter(today, &early_tomorrow_event),
            "Early tomorrow event should not match today's date"
        );
        assert!(
            time_filter(tomorrow, &early_tomorrow_event),
            "Early tomorrow event should match tomorrow's date"
        );
    }
}
