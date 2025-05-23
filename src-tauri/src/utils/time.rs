use chrono::{Local, TimeZone, NaiveDateTime, NaiveTime};

#[allow(dead_code)]
// 添加一个辅助函数来格式化时间戳
pub fn date(timestamp_millis: u64) -> String {
    // 将毫秒时间戳转换为 DateTime 对象
    let datetime = Local
        .timestamp_millis_opt(timestamp_millis as i64)
        .single()
        .unwrap_or_else(|| Local::now());

    // 格式化为 "月.日" 格式
    datetime.format("%-m.%-d %a").to_string()
}

#[allow(dead_code)]
pub fn time(timestamp_millis: u64) -> String {
    // 将毫秒时间戳转换为 DateTime 对象
    let datetime = Local
        .timestamp_millis_opt(timestamp_millis as i64)
        .single()
        .unwrap_or_else(|| Local::now());

    // 格式化为 "时:分" 格式
    datetime.format("%H:%M").to_string()
}

pub fn target_date_timestamp(days_offset:i64) -> u64 {
    // 获取距当前offset天的时间戳（毫秒）
    let now = chrono::Local::now();
    let target_date = now.date_naive() + chrono::Duration::days(days_offset);
    let time = NaiveTime::from_hms_opt(0, 0, 0).unwrap();
    let date_time = NaiveDateTime::new(target_date, time);

    // 处理时区问题
    let lacal_dt = chrono::Local.from_local_datetime(&date_time).unwrap();
    let utc_dt = lacal_dt.with_timezone(&chrono::Utc);
    let timestamp = (utc_dt.timestamp() as u64) * 1000;
    timestamp
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_date() {
        let timestamp = 1745752031000; // Example timestamp
        let formatted_date = date(timestamp);
        println!("Formatted date: {}", formatted_date);
    }

    #[test]
    fn test_time() {
        let timestamp = 1745752031000; // Example timestamp
        let formatted_time = time(timestamp);
        println!("Formatted time: {}", formatted_time);
    }
}
