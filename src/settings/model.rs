use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::utils::default_output_path;

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
#[serde(default, rename_all = "camelCase")]
pub struct SoundcloudSettings {
    pub profile_url: String,
    pub sync_interval: u32,
    pub auto_sync: bool,
    pub cached_client_id: Option<String>,
}

impl Default for SoundcloudSettings {
    fn default() -> Self {
        Self {
            profile_url: String::new(),
            sync_interval: 60,
            auto_sync: true,
            cached_client_id: None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
#[serde(default, rename_all = "camelCase")]
pub struct DownloadSettings {
    pub output_path: String,
    pub max_concurrent: u32,
    pub naming_template: String,
}

impl Default for DownloadSettings {
    fn default() -> Self {
        Self {
            output_path: default_output_path(),
            max_concurrent: 3,
            naming_template: "{artist} - {title}".to_string(),
        }
    }
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

impl AdbSettings {
    pub fn find_enabled_device(&self, device_id: &str) -> Option<&AdbDeviceSettings> {
        self.devices
            .iter()
            .find(|d| d.enabled && d.device_id == device_id)
    }
}

fn default_max_concurrent_proxies() -> usize {
    10
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct NetworkSettings {
    pub use_proxy: bool,
    pub proxy_url: String,
    #[serde(default)]
    pub fallback_proxies: Vec<String>,
    #[serde(default = "default_max_concurrent_proxies")]
    pub max_concurrent_proxies: usize,
}

impl Default for NetworkSettings {
    fn default() -> Self {
        Self {
            use_proxy: false,
            proxy_url: String::new(),
            fallback_proxies: Vec::new(),
            max_concurrent_proxies: default_max_concurrent_proxies(),
        }
    }
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
#[serde(default, rename_all = "camelCase")]
pub struct SystemSettings {
    pub limit_per_page: u32,
    pub max_retries: u32,
}

impl Default for SystemSettings {
    fn default() -> Self {
        Self {
            limit_per_page: 100,
            max_retries: 5,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, ToSchema)]
#[serde(default, rename_all = "camelCase")]
pub struct WebDavSettings {
    pub base_url: String,
    pub remote_dir: String,
    pub username: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, ToSchema)]
#[serde(default, rename_all = "camelCase")]
pub struct UserSettings {
    #[serde(default)]
    pub soundcloud: SoundcloudSettings,
    #[serde(default)]
    pub downloads: DownloadSettings,
    #[serde(default)]
    pub adb: AdbSettings,
    #[serde(default)]
    pub network: NetworkSettings,
    #[serde(default)]
    pub system: SystemSettings,
    #[serde(default)]
    pub webdav: WebDavSettings,
}
