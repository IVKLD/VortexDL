use std::{
    collections::{HashMap, HashSet},
    sync::Mutex,
};

use tokio::sync::{broadcast, watch};
use url::Url;

use super::types::{DownloadItem, DownloadStatus, ServerEvent};
use crate::{
    domain::{AudioFormat, DiscoveredTrack},
    infra::MutexExt,
};

#[derive(Default)]
struct ManagerState {
    tasks: HashMap<i64, DownloadItem>,
    reserved_urls: HashSet<String>,
    cancel_senders: HashMap<i64, watch::Sender<bool>>,
    canceled_ids: HashSet<i64>,
}

pub struct DownloadManager {
    state: Mutex<ManagerState>,
    events: broadcast::Sender<ServerEvent>,
}

impl Default for DownloadManager {
    fn default() -> Self {
        let (events, _) = broadcast::channel(10240);
        Self {
            state: Mutex::new(ManagerState::default()),
            events,
        }
    }
}

impl DownloadManager {
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

    pub fn add_task(&self, task: DiscoveredTrack) -> watch::Receiver<bool> {
        let id = task.id;
        let item = DownloadItem::from(task);
        let mut state = self.state.lock_unpoisoned();
        let initial = state.canceled_ids.contains(&id);
        let (tx, rx) = watch::channel(initial);
        state.cancel_senders.insert(id, tx);
        state.tasks.insert(id, item.clone());
        drop(state);

        let _ = self.events.send(ServerEvent::TrackUpdate {
            item: Box::new(item),
        });
        rx
    }

    pub fn get_cancel_receiver(&self, id: i64) -> Option<watch::Receiver<bool>> {
        let state = self.state.lock_unpoisoned();
        if state.canceled_ids.contains(&id) {
            let (_tx, rx) = watch::channel(true);
            return Some(rx);
        }
        state.cancel_senders.get(&id).map(|tx| tx.subscribe())
    }

    pub fn update_downloading(&self, id: i64) {
        let item = {
            let mut state = self.state.lock_unpoisoned();
            let Some(item) = state.tasks.get_mut(&id) else {
                return;
            };
            item.status = DownloadStatus::Downloading;
            item.progress = Some(0.0);
            item.clone()
        };
        let _ = self.events.send(ServerEvent::TrackUpdate {
            item: Box::new(item),
        });
    }

    pub fn update_progress(&self, id: i64, current: u64, total: u64) {
        let effective_total = if total > 0 { total } else { 3_500_000 };
        let progress = ((current as f64 / effective_total as f64) * 100.0).min(100.0);

        let updated_item = {
            let mut state = self.state.lock_unpoisoned();
            let Some(item) = state.tasks.get_mut(&id) else {
                return;
            };
            if item.progress.is_none_or(|p| (progress - p).abs() >= 0.5) {
                item.progress = Some(progress);
                item.status = DownloadStatus::Downloading;
                Some(item.clone())
            } else {
                None
            }
        };

        if let Some(item) = updated_item {
            let _ = self.events.send(ServerEvent::TrackUpdate {
                item: Box::new(item),
            });
        }
    }

    pub fn update_finished(
        &self,
        id: i64,
        format: AudioFormat,
        created_at: u64,
        source_url: Option<&Url>,
        size: u64,
    ) {
        let item = {
            let mut state = self.state.lock_unpoisoned();
            state.cancel_senders.remove(&id);
            state.canceled_ids.remove(&id);
            let Some(mut item) = state.tasks.remove(&id) else {
                return;
            };
            item.status = DownloadStatus::Finished;
            item.format = Some(format);
            item.created_at = Some(created_at);
            item.source_url = source_url.cloned();
            item.progress = Some(100.0);
            item.size = Some(size);
            item
        };
        let _ = self.events.send(ServerEvent::TrackUpdate {
            item: Box::new(item),
        });
    }

    pub fn update_failed(&self, id: i64, error_message: &str) {
        let item = {
            let mut state = self.state.lock_unpoisoned();
            state.cancel_senders.remove(&id);
            state.canceled_ids.remove(&id);
            let Some(mut item) = state.tasks.remove(&id) else {
                return;
            };
            item.status = DownloadStatus::Failed;
            item.error = Some(error_message.to_string());
            item
        };
        let _ = self.events.send(ServerEvent::TrackUpdate {
            item: Box::new(item),
        });
    }

    pub fn remove_task(&self, id: i64) {
        let item = {
            let mut state = self.state.lock_unpoisoned();
            state.canceled_ids.insert(id);
            if let Some(tx) = state.cancel_senders.remove(&id) {
                let _ = tx.send(true);
            }
            state.canceled_ids.remove(&id);
            let Some(mut item) = state.tasks.remove(&id) else {
                return;
            };
            item.status = DownloadStatus::Canceled;
            item.error = None;
            item
        };
        let _ = self.events.send(ServerEvent::TrackUpdate {
            item: Box::new(item),
        });
    }

    pub fn broadcast_event(&self, event: ServerEvent) {
        let _ = self.events.send(event);
    }

    pub fn get_queue(&self) -> Vec<DownloadItem> {
        self.state
            .lock_unpoisoned()
            .tasks
            .values()
            .filter(|t| t.is_active())
            .cloned()
            .collect()
    }

    pub fn is_idle(&self) -> bool {
        let s = self.state.lock_unpoisoned();
        s.tasks.is_empty() && s.reserved_urls.is_empty()
    }

    pub fn subscribe(&self) -> broadcast::Receiver<ServerEvent> {
        self.events.subscribe()
    }
}
