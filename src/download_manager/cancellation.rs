use std::{
    collections::{HashMap, HashSet},
    sync::Mutex,
};

use tokio::sync::watch;

use crate::utils::MutexExt;

#[derive(Default)]
struct CancellationInner {
    senders: HashMap<i64, watch::Sender<bool>>,
    canceled_ids: HashSet<i64>,
}

/// CancellationRegistry manages task cancellation channels using tokio watch signals.
#[derive(Default)]
pub struct CancellationRegistry {
    inner: Mutex<CancellationInner>,
}

impl CancellationRegistry {
    pub fn register(&self, id: i64) -> watch::Receiver<bool> {
        let mut inner = self.inner.lock_unpoisoned();
        let initial = inner.canceled_ids.contains(&id);
        let (tx, rx) = watch::channel(initial);
        inner.senders.insert(id, tx);
        rx
    }

    pub fn get_receiver(&self, id: i64) -> Option<watch::Receiver<bool>> {
        let inner = self.inner.lock_unpoisoned();
        if inner.canceled_ids.contains(&id) {
            let (_tx, rx) = watch::channel(true);
            return Some(rx);
        }
        inner.senders.get(&id).map(|tx| tx.subscribe())
    }

    pub fn trigger_cancel(&self, id: i64) {
        let mut inner = self.inner.lock_unpoisoned();
        inner.canceled_ids.insert(id);
        if let Some(tx) = inner.senders.get(&id) {
            let _ = tx.send(true);
        }
    }

    pub fn unregister(&self, id: i64) {
        let mut inner = self.inner.lock_unpoisoned();
        inner.senders.remove(&id);
        inner.canceled_ids.remove(&id);
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
