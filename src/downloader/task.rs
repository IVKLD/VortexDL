use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use soundcloud_rs::Client;
use tokio::sync::RwLock;

use crate::{
    download_manager::DownloadManager, settings::SettingsManager, storage::MusicStorage,
    types::DiscoveredMusicTrack, utils::filename::clean_filename,
};

#[derive(Clone)]
pub struct Context {
    pub storage: Arc<RwLock<MusicStorage>>,
    pub client: Arc<Client>,
    pub dm: Option<Arc<DownloadManager>>,
    pub settings: SettingsManager,
}

impl Context {
    pub fn new(
        storage: Arc<RwLock<MusicStorage>>,
        client: Arc<Client>,
        dm: Option<Arc<DownloadManager>>,
        settings: SettingsManager,
    ) -> Self {
        Self {
            storage,
            client,
            dm,
            settings,
        }
    }
}

pub struct DownloadTask {
    pub track: DiscoveredMusicTrack,
    pub file_path: PathBuf,
}

impl DownloadTask {
    pub fn display_name(&self) -> String {
        format!("{} - {}", self.track.artist, self.track.title)
    }

    pub fn new(track: &DiscoveredMusicTrack, naming_template: &str, output_dir: &Path) -> Self {
        let formatted = naming_template
            .replace("{artist}", &track.artist)
            .replace("{title}", &track.title);
        let filename = format!("{}.mp3", clean_filename(&formatted));

        Self {
            track: track.clone(),
            file_path: output_dir.join(filename),
        }
    }
}
