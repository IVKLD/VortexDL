use std::{time::Duration};

use anyhow::Result;
use soundcloud_rs::{Client, ClientBuilder};

use crate::{
    database::update_settings,
    settings::UserSettings,
    types::discovery::{ResolveQuery, ResolveResponse},
};

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

pub async fn init_client(settings: &mut UserSettings) -> Result<Client> {
    let client = init_client_with_settings(settings, None).await?;

    if settings.soundcloud.cached_client_id.is_none() {
        let current_client_id = client.get_client_id_value().await;
        settings.soundcloud.cached_client_id = Some(current_client_id);
        update_settings(settings).ok();
    }

    Ok(client)
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
