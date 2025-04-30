// 标准库导入
use std::ops::DerefMut;

// 外部库导入
use tauri::State;

// 项目内部导入
use crate::entity::{
    Event, FEvent, List, Tag, EventMetadata  ,    // 数据类型
    Entity, Repository, Storage, StorageState  // 存储和实体接口
};
use crate::utils::time;

pub fn event_to_fevent(event: &Event) -> FEvent {
    FEvent {
        id: event.metadata.uuid.clone(),
        listid: match event.metadata.list {
            None => "Undefined".to_string(),
            Some(listid) => listid.to_string(),
        },
        time: match event.task_time {
            None => "Undefined".to_string(),
            Some(time) => time::time(time),
        },
        date: match event.task_time {
            None => "Undefined".to_string(),
            Some(time) => time::date(time),
        },
        tag: event.metadata.tag.clone(),
        title: event.title.clone(),
        create: time::date(event.metadata.timestamp),
        finished: event.finished,
        priority: event.priority.clone(),
        color: event.color.clone(),
        icon: event.icon.clone(),
    }
}

fn exists<T>(state: &State<'_, StorageState>, id: &str) -> bool
where
    T: Entity + 'static,
{
    let mut guard = state.0.lock().unwrap();
    let storage = guard.deref_mut();

    let item: Option<T> = Repository::<T>::get_by_name(storage, &id.to_string())
        .map_err(|e| e.to_string())
        .unwrap();

    item.is_some()
}

pub fn tag_exists(state: &State<'_, StorageState>, name: &str) -> bool {
    exists::<Tag>(state, name)
}

pub fn list_exists(state: &State<'_, StorageState>, name: &str) -> bool {
    exists::<List>(state, name)
}

#[allow(dead_code)]
// 这一段代码是函数式编程的风格，写在这里装个逼
pub fn with_storage<F, T>(state: &State<'_, StorageState>, f: F) -> Result<T, String>
where
    F: FnOnce(&mut Storage) -> Result<T, String>,
{
    let mut guard = state.0.lock().unwrap();
    let storage = guard.deref_mut();
    f(storage)
}