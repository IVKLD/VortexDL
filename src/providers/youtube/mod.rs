pub mod client;
pub mod discovery;
pub mod download;
pub mod resolve;
pub mod search;

use std::sync::Arc;

pub use client::ClientBuilder;
use yt_audio_downloader::Client;

use crate::{infra::net::HttpClientPool, settings::SettingsManager};

#[derive(Clone)]
pub struct YouTubeProvider {
    client: Arc<Client>,
    settings: SettingsManager,
    http_clients: HttpClientPool,
}

impl YouTubeProvider {
    pub fn new(
        client: Arc<Client>,
        settings: SettingsManager,
        http_clients: HttpClientPool,
    ) -> Self {
        Self {
            client,
            settings,
            http_clients,
        }
    }
}
