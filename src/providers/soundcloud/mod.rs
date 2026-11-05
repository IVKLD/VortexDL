pub mod client;
pub mod discovery;
pub mod download;
pub mod resolve;
pub mod search;

use std::sync::Arc;

pub use client::ClientBuilder;
use soundcloud_rs::Client;

use crate::{infra::net::HttpClientPool, providers::ProviderError, settings::SettingsManager};

#[derive(Clone)]
pub struct SoundCloudProvider {
    client: Arc<Client>,
    settings: SettingsManager,
    http_clients: HttpClientPool,
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

    pub async fn sync_client_id(&self) -> Result<(), ProviderError> {
        let active_client_id = self.client.get_client_id_value().await;
        let mut current_settings = self.settings.read().await.clone();
        if current_settings.soundcloud.cached_client_id.as_ref() != Some(&active_client_id) {
            current_settings.soundcloud.cached_client_id = Some(active_client_id);
            self.settings.update(current_settings).await?;
        }
        Ok(())
    }
}
