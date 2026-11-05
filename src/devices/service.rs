use std::collections::HashSet;

use crate::{
    devices::{
        adb,
        error::DeviceError,
        players,
        storage::{StorageInfo, StorageType, get_device_storages},
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
        adb::list_devices().await
    }

    pub async fn get_device_storages(&self, device: &str) -> Result<Vec<StorageInfo>, DeviceError> {
        let mut storages = get_device_storages(device)
            .await
            .map_err(DeviceError::Other)?;

        let configured_path = {
            let settings = self.settings.read().await;
            settings
                .adb
                .devices
                .iter()
                .find(|d| d.device_id == device)
                .map(|d| d.remote_music_dir.clone())
        };

        if let Some(custom_path) = configured_path
            && !custom_path.is_empty()
            && !storages.iter().any(|s| s.path == custom_path)
        {
            let is_sd = custom_path.contains("/storage/") && !custom_path.contains("/emulated/");
            storages.insert(
                0,
                StorageInfo {
                    name: format!("Configured ({custom_path})"),
                    path: custom_path,
                    storage_type: if is_sd {
                        StorageType::SdCard
                    } else {
                        StorageType::Internal
                    },
                },
            );
        }

        Ok(storages)
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
        players::scan_device(device).await
    }

    pub async fn scan_all_players(&self) -> Result<(), DeviceError> {
        let devices = self.list_devices().await?;
        if devices.is_empty() {
            tracing::info!("No connected ADB devices found.");
            return Ok(());
        }

        for id in &devices {
            tracing::info!("Scanning music library on device: {id}...");
            self.scan_device(id).await?;
            tracing::info!("Scan triggered successfully on device: {id}");
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
