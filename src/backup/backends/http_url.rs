use std::path::Path;

use reqwest::{Client, StatusCode};
use tracing::{debug, instrument};

use super::super::{backend::BackupBackend, error::BackupError};

pub struct HttpUrlBackend {
    client: Client,
    url: String,
}

impl HttpUrlBackend {
    pub fn new(url: impl Into<String>) -> Result<Self, BackupError> {
        let client = Client::builder().build()?;
        Ok(Self {
            client,
            url: url.into(),
        })
    }
}

impl BackupBackend for HttpUrlBackend {
    async fn upload(&self, _src: &Path) -> Result<(), BackupError> {
        Err(BackupError::OperationNotSupported(
            "HttpUrlBackend does not support upload".to_string(),
        ))
    }

    #[instrument(skip(self))]
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
                debug!(
                    "URL download complete from {} ({} bytes)",
                    self.url,
                    bytes.len()
                );
                Ok(())
            }
        }
    }
}
