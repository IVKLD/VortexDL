use std::{collections::HashSet, sync::LazyLock};

use tokio::sync::Mutex;

pub static CONNECTED_DEVICES: Mutex<Option<HashSet<String>>> = Mutex::const_new(None);

pub static ACTIVE_SYNCS: LazyLock<std::sync::Mutex<HashSet<String>>> =
    LazyLock::new(|| std::sync::Mutex::new(HashSet::new()));

pub struct SyncGuard {
    pub device_id: String,
}

impl SyncGuard {
    pub fn try_acquire(device_id: &str) -> Option<Self> {
        let mut active = match ACTIVE_SYNCS.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        if active.contains(device_id) {
            None
        } else {
            active.insert(device_id.to_string());
            Some(Self {
                device_id: device_id.to_string(),
            })
        }
    }
}

impl Drop for SyncGuard {
    fn drop(&mut self) {
        let mut active = match ACTIVE_SYNCS.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        active.remove(&self.device_id);
    }
}
