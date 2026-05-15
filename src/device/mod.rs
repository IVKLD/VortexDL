use std::{sync::Arc, time::Duration};

use tokio::sync::RwLock;

use crate::{database::settings::UserSettings, storage::MusicStorage};

pub mod adb;

pub fn init(storage: Arc<RwLock<MusicStorage>>, settings: Arc<RwLock<UserSettings>>) {
    tokio::spawn(async move {
        loop {
            if let Err(e) = adb::check_devices(storage.clone(), settings.clone()).await {
                tracing::error!(error = %e, "ADB device check failed");
            }
            tokio::time::sleep(Duration::from_secs(3)).await;
        }
    });
}
