use std::collections::HashSet;

use crate::{
    devices::{
        error::DeviceError,
        list, scan,
        storage::{StorageInfo, get_device_storages},
        sync,
    },
    library::LibraryService,
    settings::SettingsManager,
};

#[derive(Clone)]
pub struct DeviceService {
    library: LibraryService,
    settings: SettingsManager,
}

impl DeviceService {
    pub fn new(library: LibraryService, settings: SettingsManager) -> Self {
        Self { library, settings }
    }

    pub async fn list_devices(&self) -> Result<HashSet<String>, DeviceError> {
        list::list_devices().await
    }

    pub async fn get_device_storages(&self, device: &str) -> Result<Vec<StorageInfo>, DeviceError> {
        get_device_storages(device)
            .await
            .map_err(DeviceError::Other)
    }

    pub async fn sync_device(
        &self,
        device: &str,
        remote_dir: &str,
        force: bool,
    ) -> Result<(), DeviceError> {
        sync::sync_device(device, remote_dir, self.library.clone(), force).await
    }

    pub async fn scan_device(&self, device: &str) -> Result<(), DeviceError> {
        scan::scan_device(device).await
    }

    pub async fn scan_all_players(&self) -> Result<(), DeviceError> {
        let devices = self.list_devices().await?;
        if devices.is_empty() {
            tracing::info!("No connected ADB devices found.");
            return Ok(());
        }

        for id in &devices {
            tracing::info!("Scanning music library on device: {id}...");
            match self.scan_device(id).await {
                Ok(()) => tracing::info!("Scan triggered successfully on device: {id}"),
                Err(e) => tracing::error!("Failed to scan device {id}: {e}"),
            }
        }

        Ok(())
    }

    pub async fn sync_configured_players(&self) -> Result<(), DeviceError> {
        let settings = self.settings.read().await;
        if !settings.adb.enabled {
            return Err(DeviceError::Other(anyhow::anyhow!(
                "ADB is disabled in settings. Please enable it in settings to sync."
            )));
        }

        let devices = self.list_devices().await?;
        if devices.is_empty() {
            tracing::info!("No connected ADB devices found.");
            return Ok(());
        }

        let mut synced_any = false;
        for id in &devices {
            if let Some(cfg) = settings.adb.find_enabled_device(id) {
                tracing::info!("Syncing with device: {} -> {}", id, cfg.remote_music_dir);
                self.sync_device(id, &cfg.remote_music_dir, true).await?;
                synced_any = true;
            } else {
                tracing::warn!(
                    "Device {} is connected but either disabled or not configured in settings.",
                    id
                );
            }
        }

        if !synced_any {
            tracing::info!("No configured and enabled ADB devices are currently connected.");
        }

        Ok(())
    }

    pub async fn sync_connected(&self) {
        sync::sync_connected(self.library.clone(), self.settings.clone()).await;
    }

    pub fn init_tracker(&self) {
        sync::init_device_tracker(self.library.clone(), self.settings.clone());
    }
}
