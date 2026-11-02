use std::{
    collections::{HashMap, HashSet},
    sync::{LazyLock, Mutex},
    time::Instant,
};

use crate::infra::MutexExt;

#[derive(Default)]
struct DeviceRegistryInner {
    connected: HashSet<String>,
    active_syncs: HashSet<String>,
    last_sync_times: HashMap<String, Instant>,
}

static REGISTRY: LazyLock<Mutex<DeviceRegistryInner>> =
    LazyLock::new(|| Mutex::new(DeviceRegistryInner::default()));

pub fn get_connected_devices() -> HashSet<String> {
    REGISTRY.lock_unpoisoned().connected.clone()
}

pub fn clear_connected_devices() {
    REGISTRY.lock_unpoisoned().connected.clear();
}

pub fn update_connected_state(current: &HashSet<String>) -> (Vec<String>, Vec<String>) {
    let mut inner = REGISTRY.lock_unpoisoned();
    let connected = current.difference(&inner.connected).cloned().collect();
    let disconnected = inner.connected.difference(current).cloned().collect();
    inner.connected = current.clone();
    (connected, disconnected)
}

pub fn get_last_sync_time(device_id: &str) -> Option<Instant> {
    REGISTRY
        .lock_unpoisoned()
        .last_sync_times
        .get(device_id)
        .copied()
}

pub fn update_last_sync_time(device_id: &str) {
    REGISTRY
        .lock_unpoisoned()
        .last_sync_times
        .insert(device_id.to_string(), Instant::now());
}

pub struct SyncGuard(String);

impl SyncGuard {
    pub fn try_acquire(device_id: &str) -> Option<Self> {
        let mut inner = REGISTRY.lock_unpoisoned();
        inner
            .active_syncs
            .insert(device_id.to_string())
            .then(|| Self(device_id.to_string()))
    }
}

impl Drop for SyncGuard {
    fn drop(&mut self) {
        REGISTRY.lock_unpoisoned().active_syncs.remove(&self.0);
    }
}
