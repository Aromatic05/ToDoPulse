use anyhow::Result;
use chrono::Utc;
use redb::{self, TableDefinition};
use serde::{Deserialize, Serialize};
use std::fs;
use std::ops::DerefMut;
use std::path::PathBuf;
use tauri::State;
use ts_rs::TS;
use uuid::Uuid;

use super::{Entity, Repository, StorageState};
use crate::error::ErrorKind;
use crate::filter::{map_filter, Filter};
use crate::function::gen_tag;
use crate::utils::{AppPaths, EVENT_CONTENT_CACHE, EVENT_LIST_CACHE};

type Table = TableDefinition<'static, &'static [u8], &'static [u8]>;

const EVENT_TABLE: Table = TableDefinition::new("events");

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct EventMetadata {
    pub uuid: String,
    pub timestamp: u64,
    pub list: Option<String>,
    pub tag: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Time {
    pub date: String,
    pub time: String,
}

impl EventMetadata {
    pub fn new() -> Self {
        Self {
            uuid: Uuid::new_v4().to_string(),
            timestamp: Utc::now().timestamp_millis() as u64,
            tag: None,
            list: None,
        }
    }
}

#[derive(Serialize, Deserialize, TS, Clone)]
pub enum Priority {
    Low,
    Medium,
    High,
    Undefined,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Event {
    pub metadata: EventMetadata,
    pub title: String,
    pub content: String,
    pub task_time: Option<u64>, //这里使用的是毫秒时间戳
    pub finished: bool,
    pub priority: Priority,
    pub icon: String,
    pub color: String,
}

#[cfg(test)]
impl Event {
    pub fn new(title: &str, content: &str) -> Self {
        let metadata = EventMetadata::new();
        Self {
            metadata,
            title: title.to_string(),
            content: content.to_string(),
            task_time: None,
            finished: false,
            priority: Priority::Undefined,
            icon: "default".to_string(),
            color: "default".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, TS, Clone)]
#[ts(export)]
pub struct FEvent {
    pub id: String,
    pub listid: String,
    pub tag: Option<Vec<String>>,
    pub title: String,
    pub create: String,
    pub ddl: String,
    pub finished: bool,
    pub priority: Priority,
    pub icon: String,
    pub color: String,
}

impl From<Event> for FEvent {
    fn from(event: Event) -> Self {
        FEvent {
            id: event.metadata.uuid,
            listid: match event.metadata.list {
                None => "Undefined".to_string(),
                Some(listid) => listid,
            },
            ddl: match event.task_time {
                None => "Undefined".to_string(),
                Some(time) => time.to_string(),
            },
            tag: event.metadata.tag,
            title: event.title,
            create: event.metadata.timestamp.to_string(),
            finished: event.finished,
            priority: event.priority,
            color: event.color,
            icon: event.icon,
        }
    }
}

impl Entity for Event {
    fn table_def() -> Table {
        EVENT_TABLE
    }
    fn id_bytes(&self) -> Vec<u8> {
        self.metadata.uuid.as_bytes().to_vec()
    }
    fn value(&self) -> Vec<u8> {
        serde_json::to_vec(self).unwrap_or_default() // the default value is empty
    }
}

/// 添加新事件
///
/// 创建一个新的事件项并保存到数据库中。根据提供的参数生成事件，
/// 包括创建事件的Markdown内容文件。会使用AI自动生成标签（如果启用）。
///
/// # 参数
/// * `state` - 应用状态，包含数据库访问权限
/// * `title` - 事件的标题
/// * `listid` - 可选的列表ID，指定事件所属的列表
/// * `priority` - 事件优先级
/// * `ddl` - 事件截止时间，毫秒时间戳字符串
///
/// # 返回
/// * `Result<Event, ErrorKind>` - 成功时返回创建的事件对象，失败时返回错误
#[tauri::command]
pub async fn add_event(
    state: State<'_, StorageState>,
    title: &str,
    listid: Option<&str>,
    priority: Priority,
    ddl: &str,
) -> Result<Event, ErrorKind> {
    let mut metadata = EventMetadata::new();
    metadata.list = listid.map(|id| id.to_string());
    let content_dir = AppPaths::data_dir().join(format!("{}", title));
    if !content_dir.exists() {
        fs::create_dir_all(&content_dir)?;
    }
    let content_path = content_dir.join(format!("{}.md", title));
    fs::write(&content_path, "")?;
    let mut new_event = Event {
        metadata,
        title: title.to_string(),
        content: content_path.to_string_lossy().to_string(),
        task_time: if ddl.is_empty() {
            None
        } else {
            ddl.parse::<u64>().ok()
        },
        finished: false,
        priority,
        color: "default".to_string(),
        icon: "default".to_string(),
    };
    new_event.metadata.tag = gen_tag(state.clone(), &content_path).await?;
    let mut guard = state.0.lock().await;
    let storage = guard.deref_mut();
    Repository::<Event>::add(storage, &new_event)?;

    // 使缓存失效
    if let Some(list_id) = &new_event.metadata.list {
        EVENT_LIST_CACHE.remove(list_id);
    }

    Ok(new_event.clone())
}

/// 获取事件内容
///
/// 根据事件UUID获取事件的Markdown内容。系统会首先从缓存中查找，
/// 如果缓存未命中，则从数据库获取事件信息，再读取对应的内容文件。
/// 获取后会更新缓存以提高后续访问速度。
///
/// # 参数
/// * `state` - 应用状态，包含数据库访问权限
/// * `uuid` - 事件的唯一标识符
///
/// # 返回
/// * `Result<String, ErrorKind>` - 成功时返回事件的Markdown内容，失败时返回错误
#[tauri::command]
pub async fn event_content(
    state: State<'_, StorageState>,
    uuid: &str,
) -> Result<String, ErrorKind> {
    // 从缓存中获取
    if let Some(content) = EVENT_CONTENT_CACHE.get(uuid) {
        return Ok(content);
    }

    // 缓存未命中，从数据库获取
    let mut guard = state.0.lock().await;
    let storage = guard.deref_mut();
    let event = Repository::<Event>::get_by_name(storage, uuid)?;
    if let Some(event) = event {
        let content = fs::read_to_string(&event.content)?;

        // 更新缓存
        EVENT_CONTENT_CACHE.set(uuid, content.clone());

        return Ok(content);
    }
    Ok("".to_string())
}

/// 写入事件内容
///
/// 更新指定事件的Markdown内容。系统会先从数据库获取事件信息，
/// 然后将新内容写入到对应的内容文件中，并更新缓存。
///
/// # 参数
/// * `state` - 应用状态，包含数据库访问权限
/// * `uuid` - 事件的唯一标识符
/// * `content` - 要写入的新Markdown内容
///
/// # 返回
/// * `Result<(), ErrorKind>` - 成功时返回空元组，失败时返回错误
#[tauri::command]
pub async fn write_content(
    state: State<'_, StorageState>,
    uuid: &str,
    content: String,
) -> Result<(), ErrorKind> {
    let mut guard = state.0.lock().await;
    let storage = guard.deref_mut();
    let event = Repository::<Event>::get_by_name(storage, uuid)?;
    if let Some(event) = event {
        fs::write(&event.content, &content)?;

        // 更新缓存
        EVENT_CONTENT_CACHE.set(uuid, content);

        return Ok(());
    }
    Err(ErrorKind::NotFound)
}

/// 更新事件
///
/// 根据前端提供的事件数据更新现有事件。系统会先从数据库获取旧事件，
/// 然后用新数据更新各个字段，并保存到数据库中。同时会清除相关缓存。
///
/// # 参数
/// * `state` - 应用状态，包含数据库访问权限
/// * `f_event` - 前端事件对象，包含要更新的事件数据
///
/// # 返回
/// * `Result<(), ErrorKind>` - 成功时返回空元组，失败时返回错误
#[tauri::command]
pub async fn update_event(
    state: State<'_, StorageState>,
    f_event: FEvent,
) -> Result<(), ErrorKind> {
    let mut guard = state.0.lock().await;
    let storage = guard.deref_mut();
    let old_event = Repository::<Event>::get_by_name(storage, &f_event.id)?;
    if let Some(mut new) = old_event {
        // 保存旧列表ID用于缓存失效
        let old_list_id = new.metadata.list.clone();

        new.metadata.tag = f_event.tag;
        new.title = f_event.title;
        new.task_time = f_event.ddl.parse::<u64>().ok();
        new.finished = f_event.finished;
        new.priority = f_event.priority;
        new.color = f_event.color;
        new.icon = f_event.icon;
        new.metadata.list = Some(f_event.listid.clone()); // 直接使用字符串类型的listid
        Repository::<Event>::add(storage, &new)?;

        // 使缓存失效
        if let Some(list_id) = old_list_id {
            EVENT_LIST_CACHE.remove(&list_id);
        }
        EVENT_LIST_CACHE.remove(&f_event.listid);

        return Ok(());
    }
    Err(ErrorKind::NotFound)
}

/// 删除事件
///
/// 根据UUID删除指定事件。系统会先获取事件信息以确定其所属列表，
/// 然后从数据库中删除事件，并清除相关缓存。
///
/// # 参数
/// * `state` - 应用状态，包含数据库访问权限
/// * `uuid` - 要删除的事件的唯一标识符
///
/// # 返回
/// * `Result<(), ErrorKind>` - 成功时返回空元组，失败时返回错误
#[tauri::command]
pub async fn delete_event(state: State<'_, StorageState>, uuid: &str) -> Result<(), ErrorKind> {
    // 获取事件所属的列表ID
    let mut guard = state.0.lock().await;
    let storage = guard.deref_mut();
    let event = Repository::<Event>::get_by_name(storage, uuid)?;

    // 删除事件的Markdown文件
    if let Some(event) = &event {
        let content_path = PathBuf::from(&event.content);
        if let Some(content_dir) = content_path.parent() {
            if fs::remove_dir_all(content_dir).is_err() {
                log::error!("Failed to delete directory: {}", content_dir.display());
            }
        }
    }
    // 删除事件
    Repository::<Event>::delete(storage, uuid)?;

    // 使缓存失效
    EVENT_CONTENT_CACHE.remove(uuid);
    if let Some(event) = event {
        if let Some(list_id) = event.metadata.list {
            EVENT_LIST_CACHE.remove(&list_id);
        }
    }

    Ok(())
}

/// 过滤事件
///
/// 根据提供的过滤条件字符串查询符合条件的事件。系统会解析过滤字符串为过滤枚举，
/// 然后应用对应的过滤函数来筛选事件，最后将结果转换为前端事件对象返回。
///
/// # 参数
/// * `state` - 应用状态，包含数据库访问权限
/// * `filter` - 过滤条件字符串，用于构建过滤函数
///
/// # 返回
/// * `Result<Vec<FEvent>, ErrorKind>` - 成功时返回符合条件的前端事件列表，失败时返回错误
#[tauri::command]
pub async fn filter_events(
    state: State<'_, StorageState>,
    filter: &str,
) -> Result<Vec<FEvent>, ErrorKind> {
    let mut guard = state.0.lock().await;
    let storage = guard.deref_mut();
    let filter_enum = match map_filter(filter) {
        Ok(filter_enum) => filter_enum,
        Err(e) => {
            log::error!("Error parsing filter: {}", e);
            return Err(ErrorKind::from(e));
        }
    };
    let filter_func = |event: &Event| -> bool {
        match &filter_enum {
            Filter::A(f) => f(event),
            Filter::B(f) => f(event),
        }
    };
    let res = Repository::<Event>::filter(storage, filter_func);
    match res {
        Ok(events) => Ok(events
            .into_iter()
            .map(|event| FEvent::from(event))
            .collect()),
        Err(e) => {
            return Err(ErrorKind::from(e));
        }
    }
}
