pub mod discovery;
pub mod download;
pub mod resolve;
pub mod search;

use crate::{infra::net::HttpClientPool, settings::SettingsManager};

#[derive(Clone)]
pub struct YouTubeProvider {
    settings: SettingsManager,
    http_clients: HttpClientPool,
}

impl YouTubeProvider {
    pub fn new(settings: SettingsManager, http_clients: HttpClientPool) -> Self {
        Self {
            settings,
            http_clients,
        }
    }
}
