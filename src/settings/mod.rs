pub mod manager;
pub mod types;

pub use manager::SettingsManager;
pub use types::{
    AdbDeviceSettings, AdbSettings, DownloadSettings, NetworkSettings, SoundcloudSettings,
    SystemSettings, UserSettings, WebDavSettings,
};
