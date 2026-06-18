use std::{collections::HashSet, sync::LazyLock};

use tokio::sync::Mutex;

pub static CONNECTED_DEVICES: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));

static ACTIVE_SYNCS: LazyLock<std::sync::Mutex<HashSet<String>>> =
    LazyLock::new(|| std::sync::Mutex::new(HashSet::new()));

fn lock_syncs() -> std::sync::MutexGuard<'static, HashSet<String>> {
    ACTIVE_SYNCS.lock().unwrap_or_else(|p| p.into_inner())
}

pub struct SyncGuard(String);

impl SyncGuard {
    pub fn try_acquire(device_id: &str) -> Option<Self> {
        lock_syncs()
            .insert(device_id.to_string())
            .then(|| Self(device_id.to_string()))
    }
}

impl Drop for SyncGuard {
    fn drop(&mut self) {
        lock_syncs().remove(&self.0);
    }
}
