use std::{
    collections::{HashMap, HashSet},
    sync::Mutex,
};
use url::Url;

use super::types::DownloadItem;

#[derive(Default)]
struct QueueState {
    tasks: HashMap<i64, DownloadItem>,
    reserved_urls: HashSet<String>,
}

/// TaskQueue manages in-memory download task states and URL reservations.
#[derive(Default)]
pub struct TaskQueue {
    state: Mutex<QueueState>,
}

impl TaskQueue {
    pub fn reserve_url(&self, url: &Url) -> bool {
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        state.reserved_urls.insert(url.to_string())
    }

    pub fn release_url(&self, url: &Url) {
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        state.reserved_urls.remove(url.as_str());
    }

    pub fn get_reserved_urls(&self) -> Vec<String> {
        let state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        state.reserved_urls.iter().cloned().collect()
    }

    pub fn insert(&self, item: DownloadItem) {
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        state.tasks.insert(item.id, item);
    }

    pub fn remove(&self, id: i64) -> Option<DownloadItem> {
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        state.tasks.remove(&id)
    }

    pub fn mutate<F, R>(&self, id: i64, f: F) -> Option<(DownloadItem, R)>
    where
        F: FnOnce(&mut DownloadItem) -> R,
    {
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        let item = state.tasks.get_mut(&id)?;
        let res = f(item);
        Some((item.clone(), res))
    }

    pub fn is_empty(&self) -> bool {
        let state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        state.tasks.is_empty()
    }

    pub fn active_items(&self) -> Vec<DownloadItem> {
        let state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        state
            .tasks
            .values()
            .filter(|t| t.is_active())
            .cloned()
            .collect()
    }
}
