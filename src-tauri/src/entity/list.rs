use redb::{self, TableDefinition};
use serde::{Deserialize, Serialize};
use std::ops::DerefMut;
use tauri::State;
use ts_rs::TS;
use uuid::Uuid;

use crate::entity::{Event, FEvent, Repository, StorageState};
use crate::utils::{event_to_fevent, list_exists};
use crate::cache::{EVENT_LIST_CACHE, LIST_CACHE};

use super::Entity;

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

#[tauri::command]
pub async fn new_list(
    state: State<'_, StorageState>,
    title: &str,
    icon: &str,
) -> Result<FList, String> {
    let mut guard = state.0.lock().unwrap();
    let storage = guard.deref_mut();
    let new_list = List::new(title, icon);
    Repository::<List>::add(storage, &new_list).map_err(|e| e.to_string())?;
    
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

#[tauri::command]
pub async fn delete_list(state: State<'_, StorageState>, listid: &str) -> Result<(), String> {
    let mut guard = state.0.lock().unwrap();
    let storage = guard.deref_mut();
    Repository::<List>::delete(storage, listid).map_err(|e| e.to_string())?;
    
    // 使相关缓存失效
    LIST_CACHE.clear();
    EVENT_LIST_CACHE.remove(listid);
    
    Ok(())
}

#[tauri::command]
pub async fn get_lists(state: State<'_, StorageState>) -> Result<Vec<FList>, String> {
    // 先尝试从缓存中获取
    let cache_key = "all_lists";
    if let Some(lists) = LIST_CACHE.get(cache_key) {
        return Ok(lists);
    }
    
    // 缓存未命中，从数据库获取
    let mut guard = state.0.lock().unwrap();
    let storage = guard.deref_mut();
    let lists = Repository::<List>::get_all(storage).map_err(|e| e.to_string())?;
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

#[tauri::command]
pub async fn list_content(
    state: State<'_, StorageState>,
    listid: &str,
    page: Option<u32>,
    page_size: Option<u32>,
) -> Result<Vec<FEvent>, String> {
    let page = page.unwrap_or(1);
    let page_size = page_size.unwrap_or(20);
    
    // 检查列表是否存在
    if !list_exists(&state, listid) {
        return Err("List not found".to_string());
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
    let mut guard = state.0.lock().unwrap();
    let storage = guard.deref_mut();
    let events = Repository::<Event>::filter(storage, |event| {
        event.metadata.list.as_ref().map_or(false, |list| list == listid)
    })
    .map_err(|e| e.to_string())?;
    
    let f_events: Vec<FEvent> = events
        .into_iter()
        .map(|event| event_to_fevent(&event))
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

#[tauri::command]
pub async fn rename_list(
    state: State<'_, StorageState>,
    listid: &str,
    new: &str,
) -> Result<(), String> {
    if !list_exists(&state, listid) {
        return Err("List not found".to_string());
    }
    let mut guard = state.0.lock().unwrap();
    let storage = guard.deref_mut();
    Repository::<List>::update(storage, listid, |list| {
        list.title = new.to_string();
        Ok(())
    })
    .map_err(|e| e.to_string())?;
    
    // 使列表缓存失效
    LIST_CACHE.clear();
    
    Ok(())
}
