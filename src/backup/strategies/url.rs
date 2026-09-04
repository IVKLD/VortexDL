use std::path::Path;

use reqwest::{Client, StatusCode};
use tracing::{debug, instrument};

use crate::backup::{error::BackupError, strategy::BackupStrategy};

pub struct UrlStrategy {
    client: Client,
    url: String,
}

impl UrlStrategy {
    pub fn new(url: impl Into<String>) -> Result<Self, BackupError> {
        let client = Client::builder().build()?;
        Ok(Self {
            client,
            url: url.into(),
        })
    }
}

impl BackupStrategy for UrlStrategy {
    async fn upload(&self, _src: &Path) -> Result<(), BackupError> {
        Err(BackupError::OperationNotSupported(
            "UrlStrategy does not support upload".to_string(),
        ))
    }

    #[instrument(skip(self), fields(url = %self.url))]
    async fn download(&self, dest: &Path) -> Result<(), BackupError> {
        debug!("URL download to {}", dest.display());

        let response = self.client.get(&self.url).send().await?;

        match response.status() {
            StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => Err(BackupError::Auth),
            StatusCode::NOT_FOUND => Err(BackupError::NotFound),
            s if !s.is_success() => Err(BackupError::HttpStatus(s)),
            _ => {
                let bytes = response.bytes().await?;
                tokio::fs::write(dest, &bytes).await?;
                debug!(bytes = bytes.len(), "URL download complete");
                Ok(())
            }
        }
    }
}
