pub mod manager;
pub mod model;

pub use manager::SettingsManager;
pub use model::{
    AdbDeviceSettings, AdbSettings, DownloadSettings, NetworkSettings, SoundcloudSettings,
    SystemSettings, UserSettings, WebDavSettings,
};
