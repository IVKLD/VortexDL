use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, RwLock};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DownloadStatus {
    Queued,
    Downloading,
    Finished,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadItem {
    pub id: i64,
    pub title: String,
    pub status: DownloadStatus,
    pub artwork_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ServerEvent {
    TrackUpdate {
        item: DownloadItem,
    },
    OperationStarted {
        url: String,
        kind: String,
    },
    OperationFinished {
        url: String,
        kind: String,
        status: String,
    },
    Error {
        message: String,
    },
    Message {
        message: String,
        level: String,
    },
}

pub struct DownloadManager {
    tasks: RwLock<HashMap<i64, DownloadItem>>,
    tx: broadcast::Sender<ServerEvent>,
}

impl Default for DownloadManager {
    fn default() -> Self {
        let (tx, _) = broadcast::channel(100);
        Self {
            tasks: RwLock::new(HashMap::new()),
            tx,
        }
    }
}

impl DownloadManager {
    pub async fn add_task(&self, id: i64, title: String, artwork_url: Option<String>) {
        let item = DownloadItem {
            id,
            title,
            status: DownloadStatus::Queued,
            artwork_url,
        };
        self.tasks.write().await.insert(id, item.clone());
        let _ = self.tx.send(ServerEvent::TrackUpdate { item });
    }

    pub async fn update_status(&self, id: i64, status: DownloadStatus) {
        let mut tasks = self.tasks.write().await;
        if let Some(item) = tasks.get_mut(&id) {
            item.status = status;
            let _ = self.tx.send(ServerEvent::TrackUpdate {
                item: item.clone(),
            });
        }
    }

    pub fn broadcast_event(&self, event: ServerEvent) {
        let _ = self.tx.send(event);
    }

    pub async fn get_queue(&self) -> Vec<DownloadItem> {
        self.tasks.read().await.values().cloned().collect()
    }

    pub fn subscribe(&self) -> broadcast::Receiver<ServerEvent> {
        self.tx.subscribe()
    }
}
