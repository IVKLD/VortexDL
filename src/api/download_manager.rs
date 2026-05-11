use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};
use tokio::sync::{RwLock, broadcast};

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
    pub status: DownloadStatus,
    pub artwork_url: Option<String>,
    pub format: Option<String>,
    pub created_at: Option<u64>,
    pub source_url: Option<String>,
    pub progress: Option<f64>,
    pub size: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ServerEvent {
    TrackUpdate { item: DownloadItem },
    SyncFinished,
    Error { message: String },
    Message { message: String, level: String },
}

pub struct DownloadManager {
    tasks: RwLock<HashMap<i64, DownloadItem>>,
    reserved_urls: RwLock<HashSet<String>>,
    tx: broadcast::Sender<ServerEvent>,
}

impl Default for DownloadManager {
    fn default() -> Self {
        let (tx, _) = broadcast::channel(100);
        Self {
            tasks: RwLock::new(HashMap::new()),
            reserved_urls: RwLock::new(HashSet::new()),
            tx,
        }
    }
}

impl DownloadManager {
    pub async fn reserve_url(&self, url: &str) -> bool {
        let mut reserved = self.reserved_urls.write().await;
        if reserved.contains(url) {
            return false;
        }
        reserved.insert(url.to_string());
        true
    }

    pub async fn release_url(&self, url: &str) {
        self.reserved_urls.write().await.remove(url);
    }

    pub async fn add_task(&self, id: i64, title: String, artwork_url: Option<String>) {
        let item = DownloadItem {
            id,
            title,
            status: DownloadStatus::Queued,
            artwork_url,
            format: None,
            created_at: None,
            source_url: None,
            progress: None,
            size: None,
        };
        self.tasks.write().await.insert(id, item.clone());
        self.notify_update(item);
    }

    pub async fn update_status(&self, id: i64, status: DownloadStatus) {
        let mut tasks = self.tasks.write().await;
        if let Some(item) = tasks.get_mut(&id) {
            item.status = status;
            self.notify_update(item.clone());
            self.check_queue_finished(&tasks).await;
        }
    }

    pub async fn update_finished(
        &self,
        id: i64,
        format: String,
        created_at: u64,
        source_url: Option<String>,
        size: u64,
    ) {
        let mut tasks = self.tasks.write().await;
        if let Some(item) = tasks.get_mut(&id) {
            item.status = DownloadStatus::Finished;
            item.format = Some(format);
            item.created_at = Some(created_at);
            item.source_url = source_url;
            item.progress = Some(100.0);
            item.size = Some(size);

            self.notify_update(item.clone());
            self.check_queue_finished(&tasks).await;
        }
    }

    pub async fn update_progress(&self, id: i64, current: u64, total: u64) {
        let mut tasks = self.tasks.write().await;
        if let Some(item) = tasks.get_mut(&id) {
            let progress = if total > 0 {
                (current as f64 / total as f64) * 100.0
            } else {
                0.0
            };

            if item.progress.is_none_or(|p| (progress - p).abs() > 0.5) {
                item.progress = Some(progress);
                item.status = DownloadStatus::Downloading;
                self.notify_update(item.clone());
            }
        }
    }

    pub async fn remove_task(&self, id: i64) {
        let mut tasks = self.tasks.write().await;
        if tasks.remove(&id).is_some() {
            self.check_queue_finished(&tasks).await;
        }
    }

    async fn check_queue_finished(&self, tasks: &HashMap<i64, DownloadItem>) {
        let has_active = tasks.values().any(|t| {
            matches!(
                t.status,
                DownloadStatus::Queued | DownloadStatus::Downloading
            )
        });
        if !has_active && !tasks.is_empty() {
            let _ = self.tx.send(ServerEvent::SyncFinished);
        }
    }

    pub fn broadcast_event(&self, event: ServerEvent) {
        let _ = self.tx.send(event);
    }

    fn notify_update(&self, item: DownloadItem) {
        self.broadcast_event(ServerEvent::TrackUpdate { item });
    }

    pub async fn get_queue(&self) -> Vec<DownloadItem> {
        self.tasks
            .read()
            .await
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
