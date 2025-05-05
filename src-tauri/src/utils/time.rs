use chrono::{Local, TimeZone};

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
