use std::{
    collections::{HashMap, HashSet},
    sync::{Mutex, MutexGuard},
};

use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;

use crate::types::api::AudioFormat;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DownloadStatus {
    Queued,
    Downloading,
    Finished,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadItem {
    pub id: i64,
    pub title: String,
    pub artist: String,
    pub status: DownloadStatus,
    pub artwork_url: Option<String>,
    pub format: Option<AudioFormat>,
    pub created_at: Option<u64>,
    pub source_url: Option<String>,
    pub progress: Option<f64>,
    pub size: Option<u64>,
    pub error: Option<String>,
    pub position: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ServerEvent {
    TrackUpdate { item: DownloadItem },
    SyncFinished,
    Error { message: String },
    Message { message: String, level: String },
}

struct ManagerState {
    tasks: HashMap<i64, DownloadItem>,
    reserved_urls: HashSet<String>,
}

pub struct DownloadManager {
    state: Mutex<ManagerState>,
    tx: broadcast::Sender<ServerEvent>,
}

impl Default for DownloadManager {
    fn default() -> Self {
        let (tx, _) = broadcast::channel(100);
        Self {
            state: Mutex::new(ManagerState {
                tasks: HashMap::new(),
                reserved_urls: HashSet::new(),
            }),
            tx,
        }
    }
}

impl DownloadManager {
    fn lock_state(&self) -> MutexGuard<'_, ManagerState> {
        match self.state.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        }
    }

    fn modify_task(&self, id: i64, f: impl FnOnce(&mut DownloadItem)) {
        let mut state = self.lock_state();
        if let Some(item) = state.tasks.get_mut(&id) {
            f(item);
            let snapshot = item.clone();
            self.notify_update(snapshot);
            self.check_queue_finished(&state);
        }
    }

    pub fn reserve_url(&self, url: &str) -> bool {
        self.lock_state().reserved_urls.insert(url.to_string())
    }

    pub fn release_url(&self, url: &str) {
        self.lock_state().reserved_urls.remove(url);
    }

    pub fn add_task(
        &self,
        id: i64,
        title: String,
        artist: String,
        artwork_url: Option<String>,
        position: Option<u32>,
    ) {
        let item = DownloadItem {
            id,
            title,
            artist,
            status: DownloadStatus::Queued,
            artwork_url,
            format: None,
            created_at: None,
            source_url: None,
            progress: None,
            size: None,
            error: None,
            position,
        };
        let mut state = self.lock_state();
        state.tasks.insert(id, item.clone());
        self.notify_update(item);
    }

    pub fn update_downloading(&self, id: i64) {
        self.modify_task(id, |item| {
            item.status = DownloadStatus::Downloading;
        });
    }

    pub fn update_failed(&self, id: i64, error_message: String) {
        self.modify_task(id, |item| {
            item.status = DownloadStatus::Failed;
            item.error = Some(error_message);
        });
    }

    pub fn update_finished(
        &self,
        id: i64,
        format: AudioFormat,
        created_at: u64,
        source_url: Option<String>,
        size: u64,
    ) {
        self.modify_task(id, |item| {
            item.status = DownloadStatus::Finished;
            item.format = Some(format);
            item.created_at = Some(created_at);
            item.source_url = source_url;
            item.progress = Some(100.0);
            item.size = Some(size);
        });
    }

    pub fn update_progress(&self, id: i64, current: u64, total: u64) {
        let mut state = self.lock_state();
        if let Some(item) = state.tasks.get_mut(&id) {
            let progress = if total > 0 {
                (current as f64 / total as f64) * 100.0
            } else {
                0.0
            };

            if item.progress.is_none_or(|p| (progress - p).abs() > 0.5) {
                item.progress = Some(progress);
                item.status = DownloadStatus::Downloading;
                let item_clone = item.clone();
                self.notify_update(item_clone);
            }
        }
    }

    pub fn remove_task(&self, id: i64) {
        let mut state = self.lock_state();
        if state.tasks.remove(&id).is_some() {
            self.check_queue_finished(&state);
        }
    }

    fn check_queue_finished(&self, state: &ManagerState) {
        let has_active = state.tasks.values().any(|t| {
            matches!(
                t.status,
                DownloadStatus::Queued | DownloadStatus::Downloading
            )
        });
        if !has_active && !state.tasks.is_empty() {
            let _ = self.tx.send(ServerEvent::SyncFinished);
        }
    }

    pub fn broadcast_event(&self, event: ServerEvent) {
        let _ = self.tx.send(event);
    }

    fn notify_update(&self, item: DownloadItem) {
        self.broadcast_event(ServerEvent::TrackUpdate { item });
    }

    pub fn get_queue(&self) -> Vec<DownloadItem> {
        self.lock_state()
            .tasks
            .values()
            .filter(|t| {
                matches!(
                    t.status,
                    DownloadStatus::Queued | DownloadStatus::Downloading
                )
            })
            .cloned()
            .collect()
    }

    pub fn subscribe(&self) -> broadcast::Receiver<ServerEvent> {
        self.tx.subscribe()
    }
}
