use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::{database::settings::update_settings_db, models::SyncMode};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SoundcloudSettings {
    pub profile_url: String,
    pub sync_interval: u32,
    pub auto_sync: bool,
    pub cached_client_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DownloadSettings {
    pub output_path: String,
    pub max_concurrent: u32,
    pub naming_template: String,
    pub sync_mode: SyncMode,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AdbDeviceSettings {
    pub device_id: String,
    pub remote_music_dir: String,
    pub enabled: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct AdbSettings {
    pub enabled: bool,
    pub auto_sync: bool,
    pub devices: Vec<AdbDeviceSettings>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UserSettings {
    pub soundcloud: SoundcloudSettings,
    pub downloads: DownloadSettings,
    #[serde(default)]
    pub adb: AdbSettings,
    pub limit_per_page: u32,
    pub max_retries: u32,
}

impl Default for UserSettings {
    fn default() -> Self {
        Self {
            soundcloud: SoundcloudSettings {
                profile_url: String::new(),
                sync_interval: 60,
                auto_sync: true,
                cached_client_id: None,
            },
            downloads: DownloadSettings {
                output_path: "./downloads".to_string(),
                max_concurrent: 3,
                naming_template: "{artist} - {title}".to_string(),
                sync_mode: SyncMode::Silent,
            },
            adb: AdbSettings::default(),
            limit_per_page: 100,
            max_retries: 5,
        }
    }
}

#[derive(Clone)]
pub struct SettingsManager {
    inner: Arc<RwLock<UserSettings>>,
}

impl SettingsManager {
    pub fn new(initial: UserSettings) -> Self {
        Self {
            inner: Arc::new(RwLock::new(initial)),
        }
    }

    pub async fn read(&self) -> tokio::sync::RwLockReadGuard<'_, UserSettings> {
        self.inner.read().await
    }

    pub async fn update(&self, new_settings: UserSettings) -> anyhow::Result<()> {
        update_settings_db(&new_settings)?;
        *self.inner.write().await = new_settings;
        Ok(())
    }
}
