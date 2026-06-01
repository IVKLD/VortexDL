use std::sync::Arc;

use soundcloud_rs::Client;

use crate::{api::download_manager::DownloadManager, settings::SettingsManager};

pub mod helpers;
pub mod likes;
pub mod playlist;
pub mod track;

pub use self::{
    helpers::{extract_artist, extract_title, get_likes, resolve_with_feedback, show_feedback},
    likes::fetch_likes,
    playlist::fetch_playlist,
    track::fetch_track,
};

pub struct DiscoveryContext<'a> {
    pub client: &'a Client,
    pub settings: &'a SettingsManager,
    pub dm: Option<&'a Arc<DownloadManager>>,
}
