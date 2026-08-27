use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use utoipa::ToSchema;

use crate::{database::update_settings, utils::paths::default_output_path};

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SoundcloudSettings {
    pub profile_url: String,
    pub sync_interval: u32,
    pub auto_sync: bool,
    pub cached_client_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DownloadSettings {
    pub output_path: String,
    pub max_concurrent: u32,
    pub naming_template: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AdbDeviceSettings {
    pub device_id: String,
    pub remote_music_dir: String,
    pub enabled: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AdbSettings {
    pub enabled: bool,
    pub auto_sync: bool,
    pub devices: Vec<AdbDeviceSettings>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct NetworkSettings {
    pub use_proxy: bool,
    pub proxy_url: String,
    #[serde(default)]
    pub fallback_proxies: Vec<String>,
}

impl NetworkSettings {
    pub fn get_proxy_url(&self) -> Option<&str> {
        if self.use_proxy && !self.proxy_url.is_empty() {
            Some(&self.proxy_url)
        } else {
            None
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SystemSettings {
    pub limit_per_page: u32,
    pub max_retries: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UserSettings {
    pub soundcloud: SoundcloudSettings,
    pub downloads: DownloadSettings,
    #[serde(default)]
    pub adb: AdbSettings,
    #[serde(default)]
    pub network: NetworkSettings,
    pub system: SystemSettings,
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
                output_path: default_output_path(),
                max_concurrent: 3,
                naming_template: "{artist} - {title}".to_string(),
            },
            adb: AdbSettings::default(),
            network: NetworkSettings::default(),
            system: SystemSettings {
                limit_per_page: 100,
                max_retries: 5,
            },
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

    pub fn get_proxy_url_sync(&self) -> Option<String> {
        self.inner
            .try_read()
            .ok()
            .and_then(|s| s.network.get_proxy_url().map(String::from))
    }

    pub async fn update(&self, new_settings: UserSettings) -> anyhow::Result<()> {
        update_settings(&new_settings)?;
        *self.inner.write().await = new_settings;
        Ok(())
    }

    pub async fn update_in_memory(&self, new_settings: UserSettings) {
        *self.inner.write().await = new_settings;
    }

    pub async fn output_path(&self) -> String {
        self.inner.read().await.downloads.output_path.clone()
    }
}
