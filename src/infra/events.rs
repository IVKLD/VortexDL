use tokio::sync::broadcast;

use crate::downloader::ServerEvent;

#[derive(Clone, Debug)]
pub struct EventBus {
    sender: broadcast::Sender<ServerEvent>,
}

impl Default for EventBus {
    fn default() -> Self {
        let (sender, _) = broadcast::channel(10240);
        Self { sender }
    }
}

impl EventBus {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn publish(&self, event: ServerEvent) {
        let _ = self.sender.send(event);
    }

    pub fn subscribe(&self) -> broadcast::Receiver<ServerEvent> {
        self.sender.subscribe()
    }
}
