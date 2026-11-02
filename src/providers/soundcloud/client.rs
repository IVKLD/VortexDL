use std::{future::Future, sync::Arc};

use soundcloud_rs::{
    Client, ClientBuilder as RawClientBuilder, Identifier, ProxyUrlProvider, StreamType, Track,
};

use crate::{
    infra::net,
    providers::ProviderError,
    settings::{SettingsManager, UserSettings},
};

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

    pub async fn build(self) -> Result<Client, ProviderError> {
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

        let client = builder.build().await?;

        if let Some(mgr) = self.settings_manager {
            let provider = ProxyUrlProvider(Arc::new(move || mgr.get_proxy_url_sync()));
            Ok(client.with_proxy_provider(provider))
        } else {
            Ok(client)
        }
    }

    pub async fn race_stream(
        settings: &UserSettings,
        id: i64,
    ) -> Result<(String, StreamType, Option<String>), ProviderError> {
        let settings_clone = settings.clone();
        let res = net::race_proxies(
            &settings.network.fallback_proxies,
            settings.network.max_concurrent_proxies,
            move |proxy| {
                let settings = settings_clone.clone();
                async move {
                    let client = ClientBuilder::new(&settings)
                        .with_proxy(Some(&proxy))
                        .build()
                        .await?;
                    let (url, stype) = client.resolve_stream(id).await?;
                    Ok((url, stype, Some(proxy)))
                }
            },
        )
        .await?;
        Ok(res)
    }
}

pub trait SoundcloudExt {
    fn resolve_stream_from_track(
        &self,
        track: &Track,
    ) -> impl Future<Output = Result<(String, StreamType), ProviderError>> + Send;

    fn resolve_stream(
        &self,
        id: i64,
    ) -> impl Future<Output = Result<(String, StreamType), ProviderError>> + Send;

    fn resolve_stream_url(
        &self,
        track: Option<&Track>,
        id: i64,
    ) -> impl Future<Output = Result<String, ProviderError>> + Send;
}

impl SoundcloudExt for Client {
    async fn resolve_stream_from_track(
        &self,
        track: &Track,
    ) -> Result<(String, StreamType), ProviderError> {
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

        Err(ProviderError::InvalidStream(
            "No playable stream URL found for track".to_string(),
        ))
    }

    async fn resolve_stream(&self, id: i64) -> Result<(String, StreamType), ProviderError> {
        let track = self.get_track(&Identifier::Id(id)).await?;
        self.resolve_stream_from_track(&track).await
    }

    async fn resolve_stream_url(
        &self,
        track: Option<&Track>,
        id: i64,
    ) -> Result<String, ProviderError> {
        let (url, _) = match track {
            Some(t) => self.resolve_stream_from_track(t).await?,
            None => self.resolve_stream(id).await?,
        };
        Ok(url)
    }
}
