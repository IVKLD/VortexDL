use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use tokio::sync::RwLock;

use crate::{
    api::{download_manager::DownloadManager, state::AppState},
    settings::SettingsManager,
    storage::MusicStorage,
    types::DiscoveredMusicTrack,
    utils::filename::clean_filename,
};

#[derive(Clone)]
pub struct Context {
    pub storage: Arc<RwLock<MusicStorage>>,
    pub client: Arc<soundcloud_rs::Client>,
    pub dm: Option<Arc<DownloadManager>>,
    pub settings: SettingsManager,
}

impl Context {
    pub fn from_state(state: &AppState) -> Self {
        Self {
            storage: state.storage.clone(),
            client: state.client.clone(),
            dm: Some(state.download_manager.clone()),
            settings: state.settings.clone(),
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
