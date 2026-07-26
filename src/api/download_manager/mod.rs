mod cancellation;
mod event_bus;
mod queue;
pub mod types;

pub use cancellation::CancellationRegistry;
pub use event_bus::EventBus;
pub use queue::TaskQueue;
use tokio::sync::{broadcast, watch};
pub use types::*;
use url::Url;

use crate::{api::types::AudioFormat, types::DiscoveredMusicTrack};

/// DownloadManager acts as a Facade combining TaskQueue (state), CancellationRegistry (cancellation), and EventBus (messaging).
#[derive(Default)]
pub struct DownloadManager {
    queue: TaskQueue,
    events: EventBus,
    cancellation: CancellationRegistry,
}

impl DownloadManager {
    pub fn reserve_url(&self, url: &Url) -> bool {
        self.queue.reserve_url(url)
    }

    pub fn release_url(&self, url: &Url) {
        self.queue.release_url(url)
    }

    pub fn add_task(&self, task: DiscoveredMusicTrack) -> watch::Receiver<bool> {
        let item = DownloadItem::from(task);
        let rx = self.cancellation.register(item.id);
        self.queue.insert(item.clone());
        self.events.notify_track_update(item);
        rx
    }

    pub fn get_cancel_receiver(&self, id: i64) -> Option<watch::Receiver<bool>> {
        self.cancellation.get_receiver(id)
    }

    pub fn update_downloading(&self, id: i64) {
        if let Some((item, _)) = self.queue.mutate(id, |item| {
            item.status = DownloadStatus::Downloading;
            item.progress = Some(0.0);
        }) {
            self.events.notify_track_update(item);
        }
    }

    pub fn update_progress(&self, id: i64, current: u64, total: u64) {
        let effective_total = if total > 0 { total } else { 3_500_000 };
        let progress = ((current as f64 / effective_total as f64) * 100.0).min(100.0);

        if let Some((item, should_notify)) = self.queue.mutate(id, |item| {
            if item.progress.is_none_or(|p| (progress - p).abs() >= 0.5) {
                item.progress = Some(progress);
                item.status = DownloadStatus::Downloading;
                true
            } else {
                false
            }
        }) && should_notify
        {
            self.events.notify_track_update(item);
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
        self.cancellation.unregister(id);
        if let Some(mut item) = self.queue.remove(id) {
            item.status = DownloadStatus::Finished;
            item.format = Some(format);
            item.created_at = Some(created_at);
            item.details.source_url = source_url.cloned();
            item.progress = Some(100.0);
            item.size = Some(size);
            self.events.notify_track_update(item);
        }
    }

    pub fn update_failed(&self, id: i64, error_message: &str) {
        self.cancellation.unregister(id);
        if let Some(mut item) = self.queue.remove(id) {
            item.status = DownloadStatus::Failed;
            item.error = Some(error_message.to_string());
            self.events.notify_track_update(item);
        }
    }

    pub fn remove_task(&self, id: i64) {
        self.cancellation.trigger_cancel(id);
        if let Some(mut item) = self.queue.remove(id) {
            item.status = DownloadStatus::Canceled;
            item.error = None;
            self.events.notify_track_update(item);
        }
    }

    pub fn broadcast_event(&self, event: ServerEvent) {
        self.events.broadcast(event);
    }

    pub fn get_queue(&self) -> Vec<DownloadItem> {
        self.queue.active_items()
    }

    pub fn get_reserved_urls(&self) -> Vec<String> {
        self.queue.get_reserved_urls()
    }

    pub fn is_idle(&self) -> bool {
        self.queue.is_empty() && self.queue.get_reserved_urls().is_empty()
    }

    pub fn subscribe(&self) -> broadcast::Receiver<ServerEvent> {
        self.events.subscribe()
    }
}
