use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use once_cell::sync::Lazy;

// 缓存条目
struct CacheEntry<T> {
    value: T,
    timestamp: Instant,
}

// 通用缓存实现
struct Cache<T: Clone> {
    data: HashMap<String, CacheEntry<T>>,
    ttl: Duration,
}

impl<T: Clone> Cache<T> {
    fn new(ttl_seconds: u64) -> Self {
        Cache {
            data: HashMap::new(),
            ttl: Duration::from_secs(ttl_seconds),
        }
    }

    fn get(&self, key: &str) -> Option<T> {
        if let Some(entry) = self.data.get(key) {
            if entry.timestamp.elapsed() < self.ttl {
                return Some(entry.value.clone());
            }
        }
        None
    }

    fn set(&mut self, key: &str, value: T) {
        self.data.insert(key.to_string(), CacheEntry {
            value,
            timestamp: Instant::now(),
        });
    }

    fn remove(&mut self, key: &str) {
        self.data.remove(key);
    }

    fn clear(&mut self) {
        self.data.clear();
    }
}

// 线程安全的缓存包装
pub struct SafeCache<T: Clone + Send + 'static> {
    cache: Arc<Mutex<Cache<T>>>,
}

impl<T: Clone + Send + 'static> SafeCache<T> {
    fn new(ttl_seconds: u64) -> Self {
        SafeCache {
            cache: Arc::new(Mutex::new(Cache::new(ttl_seconds))),
        }
    }

    pub fn get(&self, key: &str) -> Option<T> {
        let cache = self.cache.lock().unwrap();
        cache.get(key)
    }

    pub fn set(&self, key: &str, value: T) {
        let mut cache = self.cache.lock().unwrap();
        cache.set(key, value);
    }

    pub fn remove(&self, key: &str) {
        let mut cache = self.cache.lock().unwrap();
        cache.remove(key);
    }

    pub fn clear(&self) {
        let mut cache = self.cache.lock().unwrap();
        cache.clear();
    }
}

// 事件缓存类型定义
#[allow(dead_code)]
pub type EventCache = SafeCache<crate::entity::event::FEvent>;
pub type EventListCache = SafeCache<Vec<crate::entity::event::FEvent>>;
pub type EventContentCache = SafeCache<String>;
pub type ListCache = SafeCache<Vec<crate::entity::list::FList>>;

// 全局缓存实例
#[allow(dead_code)]
pub static EVENT_CACHE: Lazy<EventCache> = Lazy::new(|| SafeCache::new(300)); // 单个事件缓存
pub static EVENT_LIST_CACHE: Lazy<EventListCache> = Lazy::new(|| SafeCache::new(300)); // 事件列表缓存
pub static EVENT_CONTENT_CACHE: Lazy<EventContentCache> = Lazy::new(|| SafeCache::new(300)); // 事件内容缓存
pub static LIST_CACHE: Lazy<ListCache> = Lazy::new(|| SafeCache::new(300)); // 列表缓存

// 清除所有缓存
#[allow(dead_code)]
pub fn clear_all_caches() {
    EVENT_CACHE.clear();
    EVENT_LIST_CACHE.clear();
    EVENT_CONTENT_CACHE.clear();
    LIST_CACHE.clear();
}