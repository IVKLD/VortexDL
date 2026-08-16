use std::{
    collections::{HashMap, HashSet},
    sync::Mutex,
};

use tokio::sync::watch;

/// CancellationRegistry manages task cancellation channels using tokio watch signals.
#[derive(Default)]
pub struct CancellationRegistry {
    senders: Mutex<HashMap<i64, watch::Sender<bool>>>,
    canceled_ids: Mutex<HashSet<i64>>,
}

impl CancellationRegistry {
    pub fn register(&self, id: i64) -> watch::Receiver<bool> {
        let mut senders = self.senders.lock().unwrap_or_else(|e| e.into_inner());
        let canceled = self.canceled_ids.lock().unwrap_or_else(|e| e.into_inner());
        let initial = canceled.contains(&id);
        let (tx, rx) = watch::channel(initial);
        senders.insert(id, tx);
        rx
    }

    pub fn get_receiver(&self, id: i64) -> Option<watch::Receiver<bool>> {
        let canceled = self.canceled_ids.lock().unwrap_or_else(|e| e.into_inner());
        if canceled.contains(&id) {
            let (_tx, rx) = watch::channel(true);
            return Some(rx);
        }
        let senders = self.senders.lock().unwrap_or_else(|e| e.into_inner());
        senders.get(&id).map(|tx| tx.subscribe())
    }

    pub fn trigger_cancel(&self, id: i64) {
        {
            let mut canceled = self.canceled_ids.lock().unwrap_or_else(|e| e.into_inner());
            canceled.insert(id);
        }
        let senders = self.senders.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(tx) = senders.get(&id) {
            let _ = tx.send(true);
        }
    }

    pub fn unregister(&self, id: i64) {
        let mut senders = self.senders.lock().unwrap_or_else(|e| e.into_inner());
        senders.remove(&id);
        let mut canceled = self.canceled_ids.lock().unwrap_or_else(|e| e.into_inner());
        canceled.remove(&id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cancellation_registry_flow() {
        let registry = CancellationRegistry::default();
        let rx1 = registry.register(100);
        assert!(!*rx1.borrow());

        registry.trigger_cancel(100);
        assert!(*rx1.borrow());

        let rx_after = registry.get_receiver(100).unwrap();
        assert!(*rx_after.borrow());

        registry.unregister(100);
        assert!(registry.get_receiver(100).is_none());
    }

    #[test]
    fn test_cancel_before_get_receiver() {
        let registry = CancellationRegistry::default();
        registry.trigger_cancel(200);

        let rx = registry.get_receiver(200).unwrap();
        assert!(*rx.borrow());
    }
}
