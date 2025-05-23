use chrono::{TimeZone, Utc};
use std::fs;
use std::ops::DerefMut;
use tauri::State;

use crate::entity::event::Priority;
use crate::entity::{Event, Repository, StorageState};

/// 将单个 Event 添加到 ICS 文本中
fn export_event_to_ics(ics: &mut String, event: &Event) -> Result<(), String> {
    // 事件开始
    ics.push_str("BEGIN:VEVENT\r\n");

    // UID (使用 event 的 uuid)
    ics.push_str(&format!("UID:{}\r\n", event.metadata.uuid));

    // 创建时间戳
    let created = Utc
        .timestamp_millis_opt(event.metadata.timestamp as i64)
        .single()
        .ok_or_else(|| "无效的时间戳".to_string())?;
    ics.push_str(&format!("DTSTAMP:{}\r\n", created.format("%Y%m%dT%H%M%SZ")));

    // 标题
    let summary = escape_ics_field(&event.title);
    ics.push_str(&format!("SUMMARY:{}\r\n", summary));

    // 内容描述 (如果 content 是文件路径，读取文件内容)
    let description = fs::read_to_string(&event.content).unwrap_or_else(|_| event.content.clone());
    ics.push_str(&format!(
        "DESCRIPTION:{}\r\n",
        escape_ics_field(&description)
    ));

    // 开始时间
    if let Some(task_time) = event.task_time {
        let dt = Utc
            .timestamp_millis_opt(task_time as i64)
            .single()
            .ok_or_else(|| "无效的任务时间".to_string())?;
        ics.push_str(&format!("DTSTART:{}\r\n", dt.format("%Y%m%dT%H%M%SZ")));
    }

    // 优先级 (将 Priority 枚举转换为 ICS 优先级: 1=高, 5=中, 9=低)
    let priority = match event.priority {
        Priority::High => 1,
        Priority::Medium => 5,
        Priority::Low => 9,
        Priority::Undefined => 0,
    };
    ics.push_str(&format!("PRIORITY:{}\r\n", priority));

    // 状态 (完成状态)
    if event.finished {
        ics.push_str("STATUS:COMPLETED\r\n");
    } else {
        ics.push_str("STATUS:NEEDS-ACTION\r\n");
    }

    // 事件结束
    ics.push_str("END:VEVENT\r\n");

    Ok(())
}

/// 处理 ICS 字段中的特殊字符
fn escape_ics_field(text: &str) -> String {
    text.replace("\\", "\\\\")
        .replace(";", "\\;")
        .replace(",", "\\,")
        .replace("\n", "\\n")
        .replace("\r", "")
}

/// 导出多个事件为一个 ICS 文件
pub fn export_events_to_ics(events: Vec<Event>) -> Result<String, String> {
    let mut ics = String::new();

    // 日历开始
    ics.push_str("BEGIN:VCALENDAR\r\n");
    ics.push_str("VERSION:2.0\r\n");
    ics.push_str("PRODID:-//ToDoPulse//EN\r\n");

    // 添加所有指定的事件
    for event in events {
        export_event_to_ics(&mut ics, &event)?;
    }

    // 日历结束
    ics.push_str("END:VCALENDAR\r\n");

    Ok(ics)
}

/// 导出特定列表中的所有事件为 ICS 格式
pub async fn export_list_events_to_ics(
    state: State<'_, StorageState>,
    list_id: &str,
) -> Result<String, String> {
    // 在内部作用域中获取events，确保MutexGuard在作用域结束时被释放
    let events = {
        let mut guard = state.0.lock().await;
        let storage = guard.deref_mut();

        // 获取指定列表中的所有事件
        let filtered_events = Repository::<Event>::filter(storage, |event| {
            event
                .metadata
                .list
                .as_ref()
                .map_or(false, |list| list == list_id)
        })
        .map_err(|e| e.to_string())?;

        if filtered_events.is_empty() {
            return Err("列表中没有事件".to_string());
        }

        filtered_events
    };

    // 导出事件（MutexGuard已释放）
    export_events_to_ics(events)
}

/// 导出所有事件为 ICS 格式
pub async fn export_all_events_to_ics(state: State<'_, StorageState>) -> Result<String, String> {
    // 在内部作用域中获取events，确保MutexGuard在作用域结束时被释放
    let events = {
        let mut guard = state.0.lock().await;
        let storage = guard.deref_mut();

        // 获取所有事件
        let all_events = Repository::<Event>::get_all(storage).map_err(|e| e.to_string())?;

        if all_events.is_empty() {
            return Err("没有任何事件".to_string());
        }

        all_events
    };

    // 导出事件（MutexGuard已释放）
    export_events_to_ics(events)
}

/// Exports events in a date range to ICS calendar format
///
/// Filters events that fall within the specified time range and exports them
/// in ICS (iCalendar) format for use with calendar applications.
///
/// # Parameters
/// * `state` - Application state containing the database connection
/// * `start_time` - Start timestamp in milliseconds (Unix time)
/// * `end_time` - End timestamp in milliseconds (Unix time)
///
/// # Returns
/// * `Result<String, String>` - ICS content as string or error message
#[tauri::command]
pub async fn export_events_by_date_range_to_ics(
    state: State<'_, StorageState>,
    start_time: u64,
    end_time: u64,
) -> Result<String, String> {
    // 在内部作用域中获取events，确保MutexGuard在作用域结束时被释放
    let events = {
        let mut guard = state.0.lock().await;
        let storage = guard.deref_mut();

        // 过滤在时间范围内的事件
        let filtered_events = Repository::<Event>::filter(storage, |event| {
            if let Some(task_time) = event.task_time {
                task_time >= start_time && task_time <= end_time
            } else {
                false
            }
        })
        .map_err(|e| e.to_string())?;

        if filtered_events.is_empty() {
            return Err("指定时间范围内没有事件".to_string());
        }

        filtered_events
    };

    // 导出事件（MutexGuard已释放）
    export_events_to_ics(events)
}

/// 根据完成状态导出事件为 ICS 格式
pub async fn export_events_by_status_to_ics(
    state: State<'_, StorageState>,
    finished: bool,
) -> Result<String, String> {
    // 在内部作用域中获取events，确保MutexGuard在作用域结束时被释放
    let events = {
        let mut guard = state.0.lock().await;
        let storage = guard.deref_mut();

        // 过滤特定完成状态的事件
        let filtered_events =
            Repository::<Event>::filter(storage, |event| event.finished == finished)
                .map_err(|e| e.to_string())?;

        if filtered_events.is_empty() {
            let status = if finished { "已完成" } else { "未完成" };
            return Err(format!("没有{}的事件", status));
        }

        filtered_events
    };

    // 导出事件（MutexGuard已释放）
    export_events_to_ics(events)
}
