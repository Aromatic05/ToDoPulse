use std::ops::DerefMut;
use tauri::State;
use serde_json::{json, Value};
use chrono::{TimeZone, Utc};

use crate::entity::{Event, Repository, StorageState};

/// 将单个事件转换为JSON格式
fn export_event_to_json(event: &Event) -> Result<Value, String> {
    // 创建时间戳
    let created = Utc
        .timestamp_millis_opt(event.metadata.timestamp as i64)
        .single()
        .ok_or_else(|| "无效的时间戳".to_string())?;
    
    // 任务时间（如果有）
    let task_time = if let Some(time) = event.task_time {
        let dt = Utc
            .timestamp_millis_opt(time as i64)
            .single()
            .ok_or_else(|| "无效的任务时间".to_string())?;
        Some(dt.format("%Y-%m-%d %H:%M:%S").to_string())
    } else {
        None
    };
    
    // 优先级
    let priority_str = match event.priority {
        crate::entity::event::Priority::High => "高",
        crate::entity::event::Priority::Medium => "中",
        crate::entity::event::Priority::Low => "低",
        crate::entity::event::Priority::Undefined => "未定义",
    };
    
    // 创建JSON对象
    let json_event = json!({
        "id": event.metadata.uuid,
        "title": event.title,
        "created": created.format("%Y-%m-%d %H:%M:%S").to_string(),
        "task_time": task_time,
        "finished": event.finished,
        "priority": priority_str,
        "icon": event.icon,
        "color": event.color,
        "tags": event.metadata.tag,
        "list_id": event.metadata.list,
        "content": std::fs::read_to_string(&event.content).unwrap_or_else(|_| event.content.clone())
    });
    
    Ok(json_event)
}

/// 导出多个事件为JSON格式
#[tauri::command]
pub fn export_events_to_json(
    events: Vec<Event>,
) -> Result<String, String> {
    let mut json_events = Vec::new();
    
    // 添加所有指定的事件
    for event in &events {
        let json_event = export_event_to_json(event)?;
        json_events.push(json_event);
    }
    
    // 创建包含元数据的根对象
    let root = json!({
        "metadata": {
            "exported_at": Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            "count": events.len()
        },
        "events": json_events
    });
    
    // 序列化为JSON字符串
    serde_json::to_string_pretty(&root).map_err(|e| e.to_string())
}

/// 导出特定列表中的所有事件为JSON格式
#[tauri::command]
pub async fn export_list_events_to_json(state: State<'_, StorageState>, list_id: &str) -> Result<String, String> {
    // 在内部作用域中获取events，确保MutexGuard在作用域结束时被释放
    let events = {
        let mut guard = state.0.lock().unwrap();
        let storage = guard.deref_mut();
        
        // 获取指定列表中的所有事件
        let filtered_events = Repository::<Event>::filter(storage, |event| {
            if let Some(list) = event.metadata.list {
                list.to_string() == list_id
            } else {
                false
            }
        })
        .map_err(|e| e.to_string())?;
        
        if filtered_events.is_empty() {
            return Err("列表中没有事件".to_string());
        }
        
        filtered_events
    };
    
    // 导出事件（MutexGuard已释放）
    export_events_to_json(events)
}

/// 导出所有事件为JSON格式
#[tauri::command]
pub async fn export_all_events_to_json(state: State<'_, StorageState>) -> Result<String, String> {
    // 在内部作用域中获取events，确保MutexGuard在作用域结束时被释放
    let events = {
        let mut guard = state.0.lock().unwrap();
        let storage = guard.deref_mut();
        
        // 获取所有事件
        let all_events = Repository::<Event>::get_all(storage)
            .map_err(|e| e.to_string())?;
        
        if all_events.is_empty() {
            return Err("没有任何事件".to_string());
        }
        
        all_events
    };
    
    // 导出事件（MutexGuard已释放）
    export_events_to_json(events)
}

/// 根据时间范围导出事件为JSON格式
#[tauri::command]
pub async fn export_events_by_date_range_to_json(
    state: State<'_, StorageState>,
    start_time: u64,
    end_time: u64
) -> Result<String, String> {
    // 在内部作用域中获取events，确保MutexGuard在作用域结束时被释放
    let events = {
        let mut guard = state.0.lock().unwrap();
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
    export_events_to_json(events)
}

/// 根据完成状态导出事件为JSON格式
#[tauri::command]
pub async fn export_events_by_status_to_json(
    state: State<'_, StorageState>,
    finished: bool
) -> Result<String, String> {
    // 在内部作用域中获取events，确保MutexGuard在作用域结束时被释放
    let events = {
        let mut guard = state.0.lock().unwrap();
        let storage = guard.deref_mut();
        
        // 过滤特定完成状态的事件
        let filtered_events = Repository::<Event>::filter(storage, |event| {
            event.finished == finished
        })
        .map_err(|e| e.to_string())?;
        
        if filtered_events.is_empty() {
            let status = if finished { "已完成" } else { "未完成" };
            return Err(format!("没有{}的事件", status));
        }
        
        filtered_events
    };
    
    // 导出事件（MutexGuard已释放）
    export_events_to_json(events)
}