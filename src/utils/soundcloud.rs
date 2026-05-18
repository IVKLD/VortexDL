use std::{sync::Arc, time::Duration};

use anyhow::Result;
use soundcloud_rs::{Client, ClientBuilder};

use crate::{
    database::settings::update_settings_db,
    models::{ResolveQuery, ResolveResponse},
    settings::UserSettings,
};

pub async fn init_client(settings: &mut UserSettings) -> Result<Arc<Client>> {
    let mut builder = ClientBuilder::new()
        .with_max_retries(settings.max_retries)
        .with_retry_on_401(true);

    let used_cache = settings.soundcloud.cached_client_id.is_some();
    if let Some(ref cached_id) = settings.soundcloud.cached_client_id {
        builder = builder.with_client_id(cached_id.clone());
    }

    let client = Arc::new(
        builder
            .build()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to init SoundCloud client: {}", e))?,
    );

    if used_cache {
        let is_healthy = client.health_check().await;

        if !is_healthy {
            client.refresh_client_id().await.ok();
        }
    }

    let current_client_id = client.get_client_id_value().await;
    if settings.soundcloud.cached_client_id.as_ref() != Some(&current_client_id) {
        settings.soundcloud.cached_client_id = Some(current_client_id);
        update_settings_db(settings).ok();
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
