use std::time::Duration;

use anyhow::Result;
use soundcloud_rs::{Client, ClientBuilder};

use crate::{
    settings::{SettingsManager, UserSettings},
    types::core::{ResolveQuery, ResolvedResource},
};

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

pub async fn init_client_with_settings(
    settings: &UserSettings,
    proxy_url: Option<&str>,
) -> Result<Client> {
    let mut builder = ClientBuilder::new()
        .with_max_retries(settings.max_retries)
        .with_retry_on_401(true);

    if let Some(cached_id) = &settings.soundcloud.cached_client_id {
        builder = builder.with_client_id(cached_id.clone());
    }

    if let Some(proxy) = proxy_url.or_else(|| settings.network.get_proxy_url()) {
        builder = builder.with_proxy(proxy.to_string());
    }

    builder
        .build()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to build SoundCloud client: {e}"))
}

pub async fn fetch_artwork(client: &reqwest::Client, url: &str) -> Option<Vec<u8>> {
    let resp = client
        .get(url)
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .ok()?;
    resp.bytes().await.ok().map(|b| b.to_vec())
}

pub async fn resolve_url(client: &Client, url: &str) -> Result<ResolvedResource> {
    client
        .get("resolve", Some(&ResolveQuery { url }))
        .await
        .map_err(Into::into)
}
