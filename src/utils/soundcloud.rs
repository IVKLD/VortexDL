use std::time::Duration;

use anyhow::Result;
use soundcloud_rs::{Client, ClientBuilder};

use crate::{
    settings::{SettingsManager, UserSettings},
    types::core::{ResolveQuery, ResolveResponse},
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

    let proxy = proxy_url
        .map(|s| s.to_string())
        .or_else(|| settings.network.get_proxy_url().map(|s| s.to_string()));
    if let Some(p) = proxy {
        builder = builder.with_proxy(p);
    }

    builder
        .build()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to build SoundCloud client: {e}"))
}

pub async fn resolve_url(client: &Client, url: &str) -> Result<ResolveResponse> {
    let response: ResolveResponse = client
        .get(
            "resolve",
            Some(&ResolveQuery {
                url: Some(url.to_string()),
            }),
        )
        .await?;

    Ok(response)
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
