use std::{
    collections::{HashMap, HashSet},
    sync::Mutex,
};

use url::Url;

use super::types::DownloadItem;
use crate::utils::MutexExt;

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
    /// What: Atomically reserves a URL to prevent multiple concurrent downloads of the same playlist/track.
    /// Flow: Called by the API handler `/api/download` before dispatching a background download task.
    /// Why:  Prevents race conditions where simultaneous API requests attempt to process the same URL,
    ///       causing duplicate network traffic and filesystem write collisions.
    pub fn reserve_url(&self, url: &Url) -> bool {
        self.state
            .lock_unpoisoned()
            .reserved_urls
            .insert(url.to_string())
    }

    pub fn release_url(&self, url: &Url) {
        self.state
            .lock_unpoisoned()
            .reserved_urls
            .remove(url.as_str());
    }

    pub fn get_reserved_urls(&self) -> Vec<String> {
        self.state
            .lock_unpoisoned()
            .reserved_urls
            .iter()
            .cloned()
            .collect()
    }

    pub fn insert(&self, item: DownloadItem) {
        self.state.lock_unpoisoned().tasks.insert(item.id, item);
    }

    pub fn remove(&self, id: i64) -> Option<DownloadItem> {
        self.state.lock_unpoisoned().tasks.remove(&id)
    }

    pub fn mutate<F, R>(&self, id: i64, f: F) -> Option<(DownloadItem, R)>
    where
        F: FnOnce(&mut DownloadItem) -> R,
    {
        let mut state = self.state.lock_unpoisoned();
        let item = state.tasks.get_mut(&id)?;
        let res = f(item);
        Some((item.clone(), res))
    }

    pub fn is_empty(&self) -> bool {
        self.state.lock_unpoisoned().tasks.is_empty()
    }

    pub fn active_items(&self) -> Vec<DownloadItem> {
        self.state
            .lock_unpoisoned()
            .tasks
            .values()
            .filter(|t| t.is_active())
            .cloned()
            .collect()
    }
}
