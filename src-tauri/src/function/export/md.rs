use std::fs;
use std::ops::DerefMut;
use tauri::State;
use chrono::{TimeZone, Utc};

use crate::entity::{Event, Repository, StorageState};

/// 将单个事件转换为Markdown格式
fn export_event_to_md(md: &mut String, event: &Event) -> Result<(), String> {
    // 添加标题
    md.push_str(&format!("# {}\n\n", event.title));
    
    // 添加元数据
    let created = Utc
        .timestamp_millis_opt(event.metadata.timestamp as i64)
        .single()
        .ok_or_else(|| "无效的时间戳".to_string())?;
    
    md.push_str(&format!("- **ID**: {}\n", event.metadata.uuid));
    md.push_str(&format!("- **创建时间**: {}\n", created.format("%Y-%m-%d %H:%M:%S")));
    
    // 添加任务时间（如果有）
    if let Some(task_time) = event.task_time {
        let dt = Utc
            .timestamp_millis_opt(task_time as i64)
            .single()
            .ok_or_else(|| "无效的任务时间".to_string())?;
        md.push_str(&format!("- **截止时间**: {}\n", dt.format("%Y-%m-%d %H:%M:%S")));
    }
    
    // 添加优先级
    let priority_str = match event.priority {
        crate::entity::event::Priority::High => "高",
        crate::entity::event::Priority::Medium => "中",
        crate::entity::event::Priority::Low => "低",
        crate::entity::event::Priority::Undefined => "未定义",
    };
    md.push_str(&format!("- **优先级**: {}\n", priority_str));
    
    // 添加状态
    let status = if event.finished { "已完成" } else { "进行中" };
    md.push_str(&format!("- **状态**: {}\n", status));
    
    // 添加标签（如果有）
    if let Some(tags) = &event.metadata.tag {
        md.push_str("- **标签**: ");
        for (idx, tag) in tags.iter().enumerate() {
            if idx > 0 {
                md.push_str(", ");
            }
            md.push_str(&format!("`{}`", tag));
        }
        md.push_str("\n");
    }
    
    // 添加图标和颜色信息
    md.push_str(&format!("- **图标**: {}\n", event.icon));
    md.push_str(&format!("- **颜色**: {}\n\n", event.color));
    
    // 添加内容
    md.push_str("## 内容\n\n");
    let content = fs::read_to_string(&event.content).unwrap_or_else(|_| event.content.clone());
    md.push_str(&content);
    md.push_str("\n\n---\n\n");
    
    Ok(())
}

/// 导出多个事件为Markdown格式
pub fn export_events_to_md(
    events: Vec<Event>,
) -> Result<String, String> {
    let mut md = String::new();
    
    md.push_str("# 事件导出\n\n");
    md.push_str(&format!("导出时间: {}\n\n", Utc::now().format("%Y-%m-%d %H:%M:%S")));
    md.push_str("---\n\n");
    
    // 添加所有指定的事件
    for event in events {
        export_event_to_md(&mut md, &event)?;
    }
    
    Ok(md)
}

/// 导出特定列表中的所有事件为Markdown格式
pub async fn export_list_events_to_md(state: State<'_, StorageState>, list_id: &str) -> Result<String, String> {
    // 在内部作用域中获取events，确保MutexGuard在作用域结束时被释放
    let events = {
        let mut guard = state.0.lock().await;
        let storage = guard.deref_mut();
        
        // 获取指定列表中的所有事件
        let filtered_events = Repository::<Event>::filter(storage, |event| {
            event.metadata.list.as_ref().map_or(false, |list| list == list_id)
        })
        .map_err(|e| e.to_string())?;
        
        if filtered_events.is_empty() {
            return Err("列表中没有事件".to_string());
        }
        
        filtered_events
    };
    
    // 导出事件（MutexGuard已释放）
    export_events_to_md(events)
}

/// 导出所有事件为Markdown格式
pub async fn export_all_events_to_md(state: State<'_, StorageState>) -> Result<String, String> {
    // 在内部作用域中获取events，确保MutexGuard在作用域结束时被释放
    let events = {
        let mut guard = state.0.lock().await;
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
    export_events_to_md(events)
}

/// 根据时间范围导出事件为Markdown格式
#[tauri::command]
pub async fn export_events_by_date_range_to_md(
    state: State<'_, StorageState>,
    start_time: u64,
    end_time: u64
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
    export_events_to_md(events)
}

/// 根据完成状态导出事件为Markdown格式
pub async fn export_events_by_status_to_md(
    state: State<'_, StorageState>,
    finished: bool
) -> Result<String, String> {
    // 在内部作用域中获取events，确保MutexGuard在作用域结束时被释放
    let events = {
        let mut guard = state.0.lock().await;
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
    export_events_to_md(events)
}