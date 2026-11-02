pub mod client;
pub mod discovery;
pub mod download;
pub mod resolve;
pub mod search;

use std::sync::Arc;

pub use client::{ClientBuilder, SoundcloudExt};
pub use resolve::StreamSource;
use soundcloud_rs::Client;
use url::Url;

use crate::{infra::net::HttpClientPool, settings::SettingsManager};

#[derive(Clone)]
pub struct SoundCloudProvider {
    pub(crate) client: Arc<Client>,
    pub(crate) settings: SettingsManager,
    pub(crate) http_clients: HttpClientPool,
}

impl SoundCloudProvider {
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

    pub async fn sync_client_id(&self) {
        let active_client_id = self.client.get_client_id_value().await;
        let mut current_settings = self.settings.read().await.clone();
        if current_settings.soundcloud.cached_client_id.as_ref() != Some(&active_client_id) {
            current_settings.soundcloud.cached_client_id = Some(active_client_id);
            if let Err(e) = self.settings.update(current_settings).await {
                tracing::error!("Failed to save refreshed SoundCloud client ID: {e}");
            }
        }
    }

    pub fn resize_artwork_url(mut url: Url, size: &str) -> Url {
        let path = url.path();
        if path.contains("-large") {
            let new_path = path.replacen("-large", size, 1);
            url.set_path(&new_path);
        }
        url
    }
}
