use std::{
    collections::{HashMap, HashSet},
    sync::{Mutex, MutexGuard},
};

use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use url::Url;
use utoipa::ToSchema;

use crate::{api::types::AudioFormat, types::DiscoveredMusicTrack};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum DownloadStatus {
    Queued,
    Downloading,
    Finished,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum MessageLevel {
    Info,
    Warn,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DownloadTrackDetails {
    pub title: String,
    pub artist: String,
    #[schema(value_type = Option<String>)]
    pub artwork_url: Option<Url>,
    #[schema(value_type = Option<String>)]
    pub source_url: Option<Url>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DownloadItem {
    pub id: i64,
    #[serde(flatten)]
    pub details: DownloadTrackDetails,
    pub status: DownloadStatus,
    pub format: Option<AudioFormat>,
    pub created_at: Option<u64>,
    pub progress: Option<f64>,
    pub size: Option<u64>,
    pub error: Option<String>,
}

impl DownloadItem {
    pub fn is_active(&self) -> bool {
        matches!(
            self.status,
            DownloadStatus::Queued | DownloadStatus::Downloading
        )
    }
}

impl From<DiscoveredMusicTrack> for DownloadItem {
    fn from(task: DiscoveredMusicTrack) -> Self {
        Self {
            id: task.id,
            details: DownloadTrackDetails {
                title: task.title,
                artist: task.artist,
                artwork_url: task.artwork_url,
                source_url: task.permalink_url,
            },
            status: DownloadStatus::Queued,
            format: None,
            created_at: None,
            progress: None,
            size: None,
            error: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ServerEvent {
    TrackUpdate { item: Box<DownloadItem> },
    SyncFinished { url: Option<String> },
    SyncStarted { url: String },
    Error { message: String },
    Message { message: String, level: MessageLevel },
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
        self.state.lock().unwrap_or_else(|e| e.into_inner())
    }

    pub fn reserve_url(&self, url: &Url) -> bool {
        self.lock_state().reserved_urls.insert(url.to_string())
    }

    pub fn release_url(&self, url: &Url) {
        self.lock_state().reserved_urls.remove(url.as_str());
    }

    pub fn add_task(&self, task: DiscoveredMusicTrack) {
        let item = DownloadItem::from(task);
        let mut state = self.lock_state();
        state.tasks.insert(item.id, item.clone());
        self.notify_update(item);
    }

    pub fn update_downloading(&self, id: i64) {
        let updated = {
            let mut state = self.lock_state();
            let Some(item) = state.tasks.get_mut(&id) else {
                return;
            };
            item.status = DownloadStatus::Downloading;
            item.progress = Some(0.0);
            item.clone()
        };
        self.notify_update(updated);
    }

    fn finalize_task(&self, id: i64, mutator: impl FnOnce(&mut DownloadItem)) {
        let updated = {
            let mut state = self.lock_state();
            let Some(item) = state.tasks.get_mut(&id) else {
                return;
            };
            mutator(item);
            let updated = item.clone();
            state.tasks.remove(&id);
            updated
        };
        self.notify_update(updated);
    }

    pub fn update_failed(&self, id: i64, error_message: &str) {
        self.finalize_task(id, |item| {
            item.status = DownloadStatus::Failed;
            item.error = Some(error_message.to_string());
        });
    }

    pub fn update_finished(
        &self,
        id: i64,
        format: AudioFormat,
        created_at: u64,
        source_url: Option<&Url>,
        size: u64,
    ) {
        self.finalize_task(id, |item| {
            item.status = DownloadStatus::Finished;
            item.format = Some(format);
            item.created_at = Some(created_at);
            item.details.source_url = source_url.cloned();
            item.progress = Some(100.0);
            item.size = Some(size);
        });
    }

    pub fn update_progress(&self, id: i64, current: u64, total: u64) {
        let to_notify = {
            let mut state = self.lock_state();
            let Some(item) = state.tasks.get_mut(&id) else {
                return;
            };
            let progress = if total > 0 {
                (current as f64 / total as f64) * 100.0
            } else {
                0.0
            };
            if item.progress.is_none_or(|p| (progress - p).abs() > 0.5) {
                item.progress = Some(progress);
                item.status = DownloadStatus::Downloading;
                Some(item.clone())
            } else {
                None
            }
        };
        if let Some(updated) = to_notify {
            self.notify_update(updated);
        }
    }

    pub fn remove_task(&self, id: i64) {
        self.lock_state().tasks.remove(&id);
    }

    pub fn broadcast_event(&self, event: ServerEvent) {
        let _ = self.tx.send(event);
    }

    fn notify_update(&self, item: DownloadItem) {
        self.broadcast_event(ServerEvent::TrackUpdate { item: Box::new(item) });
    }

    pub fn get_queue(&self) -> Vec<DownloadItem> {
        self.lock_state()
            .tasks
            .values()
            .filter(|t| t.is_active())
            .cloned()
            .collect()
    }

    pub fn get_reserved_urls(&self) -> Vec<String> {
        self.lock_state().reserved_urls.iter().cloned().collect()
    }

    pub fn subscribe(&self) -> broadcast::Receiver<ServerEvent> {
        self.tx.subscribe()
    }
}
