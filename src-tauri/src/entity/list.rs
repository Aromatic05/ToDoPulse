use anyhow::anyhow;
use redb::{self, TableDefinition};
use serde::{Deserialize, Serialize};
use std::ops::DerefMut;
use tauri::State;
use ts_rs::TS;
use uuid::Uuid;

use crate::entity::{Event, FEvent, Repository, StorageState};
use crate::error::ErrorKind;
use crate::utils::{list_exists, EVENT_LIST_CACHE, LIST_CACHE};

use super::{Entity,event::delete_event};

type Table = TableDefinition<'static, &'static [u8], &'static [u8]>;

const LIST_TABLE: Table = TableDefinition::new("lists");

#[derive(Serialize, Deserialize, TS, Clone)]
#[ts(export)]
pub struct FList {
    pub id: String,
    pub title: String,
    pub icon: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct List {
    pub uuid: String,
    pub title: String,
    pub icon: String,
}

impl Entity for List {
    fn table_def() -> Table {
        LIST_TABLE
    }
    fn id_bytes(&self) -> Vec<u8> {
        self.uuid.as_bytes().to_vec()
    }
    fn value(&self) -> Vec<u8> {
        serde_json::to_vec(self).unwrap()
    }
}

impl List {
    pub fn new(title: &str, icon: &str) -> Self {
        let uuid = Uuid::new_v4().to_string();
        let icon = icon.to_string();
        let title = title.to_string();
        Self { title, uuid, icon }
    }
}

/// Creates a new list in the database
///
/// Creates a list with the given title and icon, generates a UUID for it,
/// and stores it in the database.
///
/// # Parameters
/// * `state` - Application state containing the database connection
/// * `title` - Title for the new list
/// * `icon` - Icon identifier for the new list
///
/// # Returns
/// * `Result<FList, ErrorKind>` - The newly created list in frontend format or an error
#[tauri::command]
pub async fn new_list(
    state: State<'_, StorageState>,
    title: &str,
    icon: &str,
) -> Result<FList, ErrorKind> {
    let mut guard = state.0.lock().await;
    let storage = guard.deref_mut();
    let new_list = List::new(title, icon);
    Repository::<List>::add(storage, &new_list)?;

    // 使列表缓存失效
    LIST_CACHE.clear();

    // 转换为前端使用的 FList 格式
    let f_list = FList {
        id: new_list.uuid.clone(),
        title: new_list.title.clone(),
        icon: new_list.icon.clone(),
    };

    Ok(f_list)
}

/// Deletes a list from the database
///
/// Removes the list with the specified ID from the database and invalidates all related caches.
/// Also deletes all events associated with the list.
///
/// # Parameters
/// * `state` - Application state containing the database connection
/// * `listid` - UUID of the list to delete
///
/// # Returns
/// * `Result<(), ErrorKind>` - Success or an error if the list couldn't be deleted
#[tauri::command]
pub async fn delete_list(state: State<'_, StorageState>, listid: &str) -> Result<(), ErrorKind> {
    // 获取锁并收集所有相关事件的UUID
    let event_uuids = {
        let mut guard = state.0.lock().await;
        let storage = guard.deref_mut();
        
        // 收集所有该列表的事件的UUID
        let events = Repository::<Event>::filter(storage, |event| {
            event.metadata.list.as_ref().map_or(false, |list| list == listid)
        })?;
        
        // 只提取UUID
        events.iter().map(|event| event.metadata.uuid.clone()).collect::<Vec<_>>()
    };
    
    // 使用futures::future::join_all进行并行删除
    let deletion_results = futures::future::join_all(
        event_uuids.iter().map(|uuid| delete_event(state.clone(), uuid))
    ).await;
    
    // 记录任何删除失败的事件
    for (i, result) in deletion_results.iter().enumerate() {
        if let Err(e) = result {
            log::error!("Failed to delete event {}: {}", event_uuids[i], e);
        }
    }
    
    // 删除列表本身
    {
        let mut guard = state.0.lock().await;
        let storage = guard.deref_mut();
        Repository::<List>::delete(storage, listid)?;
    }

    // 使相关缓存失效
    LIST_CACHE.clear();
    EVENT_LIST_CACHE.remove(listid);

    Ok(())
}

/// Retrieves all lists from the database
///
/// Fetches all lists from the database and converts them to the frontend format.
/// Uses caching to improve performance on repeated calls.
///
/// # Parameters
/// * `state` - Application state containing the database connection
///
/// # Returns
/// * `Result<Vec<FList>, ErrorKind>` - All lists in frontend format or an error
#[tauri::command]
pub async fn get_lists(state: State<'_, StorageState>) -> Result<Vec<FList>, ErrorKind> {
    // 先尝试从缓存中获取
    let cache_key = "all_lists";
    if let Some(lists) = LIST_CACHE.get(cache_key) {
        return Ok(lists);
    }

    // 缓存未命中，从数据库获取
    let mut guard = state.0.lock().await;
    let storage = guard.deref_mut();
    let lists = Repository::<List>::get_all(storage)?;
    let f_lists: Vec<FList> = lists
        .into_iter()
        .map(|list| FList {
            id: list.uuid,
            title: list.title,
            icon: list.icon,
        })
        .collect();

    // 更新缓存
    LIST_CACHE.set(cache_key, f_lists.clone());

    Ok(f_lists)
}

/// Retrieves events belonging to a specific list with pagination support
///
/// Fetches events that belong to the specified list and returns them in frontend format.
/// Supports pagination to limit the number of events returned. Uses caching to improve
/// performance on repeated calls.
///
/// # Parameters
/// * `state` - Application state containing the database connection
/// * `listid` - UUID of the list to get events from
/// * `page` - Optional page number, defaults to 1 if not specified
/// * `page_size` - Optional number of events per page, defaults to 20 if not specified
///
/// # Returns
/// * `Result<Vec<FEvent>, ErrorKind>` - Paginated events in frontend format or an error
/// * Returns error if the list doesn't exist
#[tauri::command]
pub async fn list_content(
    state: State<'_, StorageState>,
    listid: &str,
    page: Option<u32>,
    page_size: Option<u32>,
) -> Result<Vec<FEvent>, ErrorKind> {
    let page = page.unwrap_or(1);
    let page_size = page_size.unwrap_or(20);

    // 检查列表是否存在
    if !list_exists(&state, listid).await {
        return Err(anyhow!("List not found").into());
    }

    // 先尝试从缓存中获取（全量数据）
    let cached_events = EVENT_LIST_CACHE.get(listid);

    if let Some(all_events) = cached_events {
        // 计算分页
        let start = ((page - 1) * page_size) as usize;
        let end = std::cmp::min(start + page_size as usize, all_events.len());

        if start < all_events.len() {
            return Ok(all_events[start..end].to_vec());
        }
    }

    // 缓存未命中或分页超出范围，从数据库获取
    let mut guard = state.0.lock().await;
    let storage = guard.deref_mut();
    let events = Repository::<Event>::filter(storage, |event| {
        event
            .metadata
            .list
            .as_ref()
            .map_or(false, |list| list == listid)
    })?;

    let f_events: Vec<FEvent> = events
        .into_iter()
        .map(|event| FEvent::from(event))
        .collect();

    // 更新缓存
    EVENT_LIST_CACHE.set(listid, f_events.clone());

    // 计算分页
    let start = ((page - 1) * page_size) as usize;
    let end = std::cmp::min(start + page_size as usize, f_events.len());

    if start < f_events.len() {
        Ok(f_events[start..end].to_vec())
    } else {
        Ok(vec![])
    }
}

/// Renames a list
///
/// Updates the title of the specified list and invalidates relevant caches.
///
/// # Parameters
/// * `state` - Application state containing the database connection
/// * `listid` - UUID of the list to rename
/// * `new` - New title for the list
///
/// # Returns
/// * `Result<(), ErrorKind>` - Success or an error if the list couldn't be renamed
/// * Returns error if the list doesn't exist
#[tauri::command]
pub async fn rename_list(
    state: State<'_, StorageState>,
    listid: &str,
    new: &str,
) -> Result<(), ErrorKind> {
    if !list_exists(&state, listid).await {
        return Err(anyhow!("List not found").into());
    }
    let mut guard = state.0.lock().await;
    let storage = guard.deref_mut();
    Repository::<List>::update(storage, listid, |list| {
        list.title = new.to_string();
        Ok(())
    })?;

    // 使列表缓存失效
    LIST_CACHE.clear();

    Ok(())
}
