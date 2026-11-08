use std::sync::Arc;

use yt_audio_downloader::{Client, ClientBuilder as RawClientBuilder, ProxyUrlProvider};

use crate::{
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
        let mut builder =
            RawClientBuilder::new().with_max_retries(self.settings.system.max_retries);

        if let Some(proxy) = self
            .proxy_url
            .or_else(|| self.settings.network.get_proxy_url())
        {
            builder = builder.with_proxy(proxy.to_string());
        }

        if let Some(mgr) = self.settings_manager {
            let provider = ProxyUrlProvider(Arc::new(move || mgr.get_proxy_url_sync()));
            builder = builder.with_proxy_provider(provider);
        }

        let client = builder.build()?;
        Ok(client)
    }
}
