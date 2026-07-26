use tokio::sync::broadcast;

use super::types::{DownloadItem, ServerEvent};

/// EventBus handles real-time event broadcasting to connected WebSocket clients.
pub struct EventBus {
    tx: broadcast::Sender<ServerEvent>,
}

impl Default for EventBus {
    fn default() -> Self {
        let (tx, _) = broadcast::channel(10240);
        Self { tx }
    }
}

impl EventBus {
    pub fn broadcast(&self, event: ServerEvent) {
        let _ = self.tx.send(event);
    }

    pub fn notify_track_update(&self, item: DownloadItem) {
        self.broadcast(ServerEvent::TrackUpdate {
            item: Box::new(item),
        });
    }

    pub fn subscribe(&self) -> broadcast::Receiver<ServerEvent> {
        self.tx.subscribe()
    }
}
