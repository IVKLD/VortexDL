use std::sync::Arc;

use anyhow::Result;
use soundcloud_rs::{
    Client, ClientBuilder as RawClientBuilder, Identifier, ProxyUrlProvider, StreamType,
};
use url::Url;

use crate::settings::{SettingsManager, UserSettings};

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

pub struct ClientBuilder<'a> {
    settings: &'a UserSettings,
    proxy_url: Option<&'a str>,
    settings_manager: Option<SettingsManager>,
}

impl<'a> ClientBuilder<'a> {
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
        let mut builder = RawClientBuilder::new()
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

pub trait SoundcloudExt {
    fn resolve_stream_from_track(
        &self,
        track: &soundcloud_rs::Track,
    ) -> impl std::future::Future<Output = Result<(String, StreamType)>> + Send;

    fn resolve_stream(
        &self,
        id: i64,
    ) -> impl std::future::Future<Output = Result<(String, StreamType)>> + Send;

    fn resolve_stream_url(
        &self,
        track: Option<&soundcloud_rs::Track>,
        id: i64,
    ) -> impl std::future::Future<Output = Result<String>> + Send;
}

impl SoundcloudExt for Client {
    async fn resolve_stream_from_track(
        &self,
        track: &soundcloud_rs::Track,
    ) -> Result<(String, StreamType)> {
        if let Ok(url) = self
            .resolve_stream_url_from_track(track, Some(&StreamType::Progressive))
            .await
        {
            return Ok((url, StreamType::Progressive));
        }
        if let Ok(url) = self
            .resolve_stream_url_from_track(track, Some(&StreamType::Hls))
            .await
        {
            return Ok((url, StreamType::Hls));
        }

        anyhow::bail!("No playable stream URL found for track")
    }

    async fn resolve_stream(&self, id: i64) -> Result<(String, StreamType)> {
        let track = self.get_track(&Identifier::Id(id)).await?;
        self.resolve_stream_from_track(&track).await
    }

    async fn resolve_stream_url(
        &self,
        track: Option<&soundcloud_rs::Track>,
        id: i64,
    ) -> Result<String> {
        let (url, _) = match track {
            Some(t) => self.resolve_stream_from_track(t).await?,
            None => self.resolve_stream(id).await?,
        };
        Ok(url)
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
