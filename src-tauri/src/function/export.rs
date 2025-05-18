mod ics;
mod json;
mod md;
pub mod save;

use self::ics::*;
use self::json::*;
use self::md::*;

use crate::entity::{Event, Repository, StorageState};
use anyhow::Result;
use std::ops::DerefMut;
use tauri::State;
use serde_json;

/// Exports specific events in the requested format
/// 
/// Takes a list of event IDs and exports them in the specified format.
/// Supports exporting to ICS, Markdown, or JSON formats.
/// 
/// # Parameters
/// * `state` - Application state containing the database connection
/// * `event_ids` - JSON value containing event IDs to export (string or array of strings)
/// * `fmt` - Format to export to: "ics", "md", or "json"
/// 
/// # Returns
/// * `Result<String, String>` - Exported content as string or error message
#[tauri::command]
pub async fn export_events(
    state: State<'_, StorageState>,
    event_ids: serde_json::Value,
    fmt: &str,
) -> Result<String, String> {
    let mut guard = state.0.lock().await;
    let storage = guard.deref_mut();
    let event_ids: Vec<String> = if event_ids.is_string() {
        vec![event_ids.as_str().unwrap_or("").to_string()]
    } else if event_ids.is_array() {
        event_ids
            .as_array()
            .unwrap_or(&Vec::new())
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect()
    } else {
        return Err("Invalid event IDs".to_string());
    };
    let events: Vec<Event> = event_ids
        .iter()
        .filter_map(|id| match Repository::<Event>::get_by_name(storage, id) {
            Ok(Some(event)) => Some(event),
            Ok(None) => None,
            Err(e) => {
                eprintln!("获取事件 {} 时出错: {}", id, e);
                None
            }
        })
        .collect();
    match fmt {
        "ics" => export_events_to_ics(events),
        "md" => export_events_to_md(events),
        "json" => export_events_to_json(events),
        _ => Err("Unsupported format".to_string()),
    }
}

/// Exports all events from a specific list in the requested format
/// 
/// Retrieves all events belonging to the specified list and exports them
/// in the requested format.
/// 
/// # Parameters
/// * `state` - Application state containing the database connection
/// * `list_id` - UUID of the list containing events to export
/// * `fmt` - Format to export to: "ics", "md", or "json"
/// 
/// # Returns
/// * `Result<String, String>` - Exported content as string or error message
#[tauri::command]
pub async fn export_list_events(
    state: State<'_, StorageState>,
    list_id: &str,
    fmt: &str,
) -> Result<String, String> {
    match fmt {
        "ics" => export_list_events_to_ics(state, list_id).await,
        "md" => export_list_events_to_md(state, list_id).await,
        "json" => export_list_events_to_json(state, list_id).await,
        _ => Err("Unsupported format".to_string()),
    }
}
/// Exports all events in the database in the requested format
/// 
/// Retrieves all events from the database and exports them in the specified format.
/// 
/// # Parameters
/// * `state` - Application state containing the database connection
/// * `fmt` - Format to export to: "ics", "md", or "json"
/// 
/// # Returns
/// * `Result<String, String>` - Exported content as string or error message
#[tauri::command]
pub async fn export_all_events(
    state: State<'_, StorageState>,
    fmt: &str,
) -> Result<String, String> {
    match fmt {
        "ics" => export_all_events_to_ics(state).await,
        "md" => export_all_events_to_md(state).await,
        "json" => export_all_events_to_json(state).await,
        _ => Err("Unsupported format".to_string()),
    }
}

/// Exports events within a specific date range in the requested format
/// 
/// Retrieves events that fall within the specified timestamp range and
/// exports them in the requested format.
/// 
/// # Parameters
/// * `state` - Application state containing the database connection
/// * `start` - Start timestamp (Unix milliseconds)
/// * `end` - End timestamp (Unix milliseconds)
/// * `fmt` - Format to export to: "ics", "md", or "json"
/// 
/// # Returns
/// * `Result<String, String>` - Exported content as string or error message
#[tauri::command]
pub async fn export_events_by_date_range(
    state: State<'_, StorageState>,
    start: u64,
    end: u64,
    fmt: &str,
) -> Result<String, String> {
    match fmt {
        "ics" => export_events_by_date_range_to_ics(state, start, end).await,
        "md" => export_events_by_date_range_to_md(state, start, end).await,
        "json" => export_events_by_date_range_to_json(state, start, end).await,
        _ => Err("Unsupported format".to_string()),
    }
}

/// Exports events with a specific completion status in the requested format
/// 
/// Retrieves events with the specified completion status (completed or not completed)
/// and exports them in the requested format.
/// 
/// # Parameters
/// * `state` - Application state containing the database connection
/// * `status` - Completion status to filter by (true = completed, false = not completed)
/// * `fmt` - Format to export to: "ics", "md", or "json"
/// 
/// # Returns
/// * `Result<String, String>` - Exported content as string or error message
#[tauri::command]
pub async fn export_events_by_status(
    state: State<'_, StorageState>,
    status: bool,
    fmt: &str,
) -> Result<String, String> {
    match fmt {
        "ics" => export_events_by_status_to_ics(state, status).await,
        "md" => export_events_by_status_to_md(state, status).await,
        "json" => export_events_by_status_to_json(state, status).await,
        _ => Err("Unsupported format".to_string()),
    }
}
