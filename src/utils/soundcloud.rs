use std::sync::Arc;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use soundcloud_rs::{Client, ClientBuilder, ProxyUrlProvider};
use url::Url;

use crate::settings::{SettingsManager, UserSettings};

#[derive(Serialize)]
pub struct ResolveQuery<'a> {
    pub url: &'a str,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum ResolvedResource {
    Track(soundcloud_rs::Track),
    User(soundcloud_rs::User),
    Playlist(soundcloud_rs::Playlist),
}

pub async fn update_cached_client_id(client: &Client, settings: &SettingsManager) {
    let active_client_id = client.get_client_id_value().await;
    let mut current_settings = settings.read().await.clone();
    if current_settings.soundcloud.cached_client_id.as_ref() != Some(&active_client_id) {
        current_settings.soundcloud.cached_client_id = Some(active_client_id);
        if let Err(e) = settings.update(current_settings).await {
            tracing::error!("Failed to save refreshed SoundCloud client ID: {e}");
        }
    }
}

pub struct SoundCloudClientBuilder<'a> {
    settings: &'a UserSettings,
    proxy_url: Option<&'a str>,
    settings_manager: Option<SettingsManager>,
}

impl<'a> SoundCloudClientBuilder<'a> {
    pub fn new(settings: &'a UserSettings) -> Self {
        Self {
            settings,
            proxy_url: None,
            settings_manager: None,
        }
    }

    pub fn with_proxy(mut self, proxy_url: Option<&'a str>) -> Self {
        self.proxy_url = proxy_url;
        self
    }

    pub fn with_settings_manager(mut self, settings_manager: SettingsManager) -> Self {
        self.settings_manager = Some(settings_manager);
        self
    }

    pub async fn build(self) -> Result<Client> {
        let mut builder = ClientBuilder::new()
            .with_max_retries(self.settings.system.max_retries)
            .with_retry_on_401(true);

        if let Some(cached_id) = &self.settings.soundcloud.cached_client_id {
            builder = builder.with_client_id(cached_id.clone());
        }

        if let Some(proxy) = self
            .proxy_url
            .or_else(|| self.settings.network.get_proxy_url())
        {
            builder = builder.with_proxy(proxy.to_string());
        }

        let client = builder
            .build()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to build SoundCloud client: {e}"))?;

        if let Some(mgr) = self.settings_manager {
            let provider = ProxyUrlProvider(Arc::new(move || mgr.get_proxy_url_sync()));
            Ok(client.with_proxy_provider(provider))
        } else {
            Ok(client)
        }
    }
}

pub async fn resolve_url(client: &Client, url: &Url) -> Result<ResolvedResource> {
    client
        .get("resolve", Some(&ResolveQuery { url: url.as_str() }))
        .await
        .map_err(Into::into)
}

pub fn resize_artwork_url(mut url: Url, size: &str) -> Url {
    let path = url.path();
    if path.contains("-large") {
        let new_path = path.replacen("-large", size, 1);
        url.set_path(&new_path);
    }
    url
}
