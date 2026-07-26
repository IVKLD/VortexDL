use std::{collections::HashMap, sync::Mutex};

use tokio::sync::watch;

/// CancellationRegistry manages task cancellation channels using tokio watch signals.
#[derive(Default)]
pub struct CancellationRegistry {
    senders: Mutex<HashMap<i64, watch::Sender<bool>>>,
}

impl CancellationRegistry {
    pub fn register(&self, id: i64) -> watch::Receiver<bool> {
        let (tx, rx) = watch::channel(false);
        let mut senders = self.senders.lock().unwrap_or_else(|e| e.into_inner());
        senders.insert(id, tx);
        rx
    }

    pub fn get_receiver(&self, id: i64) -> Option<watch::Receiver<bool>> {
        let senders = self.senders.lock().unwrap_or_else(|e| e.into_inner());
        senders.get(&id).map(|tx| tx.subscribe())
    }

    pub fn trigger_cancel(&self, id: i64) {
        let mut senders = self.senders.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(tx) = senders.remove(&id) {
            let _ = tx.send(true);
        }
    }

    pub fn unregister(&self, id: i64) {
        let mut senders = self.senders.lock().unwrap_or_else(|e| e.into_inner());
        senders.remove(&id);
    }
}
