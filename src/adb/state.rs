use std::{
    collections::{HashMap, HashSet},
    sync::{LazyLock, Mutex, MutexGuard},
    time::Instant,
};

use crate::utils::MutexExt;

static CONNECTED_DEVICES: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));

static ACTIVE_SYNCS: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));

fn lock_syncs() -> MutexGuard<'static, HashSet<String>> {
    ACTIVE_SYNCS.lock_unpoisoned()
}

static LAST_SYNC_TIMES: LazyLock<Mutex<HashMap<String, Instant>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub fn lock_connected() -> MutexGuard<'static, HashSet<String>> {
    CONNECTED_DEVICES.lock_unpoisoned()
}

pub fn get_last_sync_time(device_id: &str) -> Option<Instant> {
    LAST_SYNC_TIMES.lock_unpoisoned().get(device_id).copied()
}

pub fn update_last_sync_time(device_id: &str) {
    LAST_SYNC_TIMES
        .lock_unpoisoned()
        .insert(device_id.to_string(), Instant::now());
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
