use std::path::Path;

use reqwest::{Client, StatusCode};
use tracing::{debug, instrument};

use crate::backup::{error::SyncError, strategy::ISyncStrategy};

pub struct UrlStrategy {
    client: Client,
    url: String,
}

impl UrlStrategy {
    pub fn new(url: impl Into<String>) -> Result<Self, SyncError> {
        let client = Client::builder().build()?;
        Ok(Self {
            client,
            url: url.into(),
        })
    }
}

impl ISyncStrategy for UrlStrategy {
    async fn upload(&self, _src: &Path) -> Result<(), SyncError> {
        Err(SyncError::OperationNotSupported(
            "UrlStrategy does not support upload".to_string(),
        ))
    }

    #[instrument(skip(self), fields(url = %self.url))]
    async fn download(&self, dest: &Path) -> Result<(), SyncError> {
        debug!("URL download to {}", dest.display());

        let response = self.client.get(&self.url).send().await?;

        match response.status() {
            StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => Err(SyncError::Auth),
            StatusCode::NOT_FOUND => Err(SyncError::NotFound),
            s if !s.is_success() => Err(SyncError::HttpStatus(s)),
            _ => {
                let bytes = response.bytes().await?;
                tokio::fs::write(dest, &bytes).await?;
                debug!(bytes = bytes.len(), "URL download complete");
                Ok(())
            }
        }
    }
}
