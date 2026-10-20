use std::path::Path;

use reqwest::{
    Client, Method, StatusCode,
    header::{AUTHORIZATION, CONTENT_TYPE},
};
use tracing::{debug, instrument};

use super::super::{backend::BackupBackend, error::BackupError};

const SNAPSHOT_FILENAME: &str = "state.json";

pub struct WebDavBackend {
    client: Client,
    file_url: String,
    dir_url: String,
    auth_header: String,
}

impl WebDavBackend {
    pub fn new(
        base_url: impl AsRef<str>,
        remote_dir: impl AsRef<str>,
        username: impl AsRef<str>,
        password: impl AsRef<str>,
    ) -> Result<Self, BackupError> {
        use base64::{Engine as _, engine::general_purpose::STANDARD};

        let client = Client::builder().build()?;

        let credentials = format!("{}:{}", username.as_ref(), password.as_ref());
        let auth_header = format!("Basic {}", STANDARD.encode(credentials.as_bytes()));

        let base = base_url.as_ref().trim_end_matches('/');
        let dir = remote_dir.as_ref().trim_matches('/');
        let dir_url = format!("{base}/{dir}");
        let file_url = format!("{dir_url}/{SNAPSHOT_FILENAME}");

        Ok(Self {
            client,
            file_url,
            dir_url,
            auth_header,
        })
    }

    async fn ensure_dir(&self) -> Result<(), BackupError> {
        let response = self
            .client
            .request(Method::from_bytes(b"MKCOL").unwrap(), &self.dir_url)
            .header(AUTHORIZATION, &self.auth_header)
            .send()
            .await?;

        match response.status() {
            s if s.is_success() => Ok(()),
            StatusCode::METHOD_NOT_ALLOWED => Ok(()),
            other => Err(Self::map_status(other)),
        }
    }

    fn map_status(status: StatusCode) -> BackupError {
        match status {
            StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => BackupError::Auth,
            StatusCode::NOT_FOUND => BackupError::NotFound,
            other => BackupError::HttpStatus(other),
        }
    }
}

impl BackupBackend for WebDavBackend {
    #[instrument(skip(self))]
    async fn upload(&self, src: &Path) -> Result<(), BackupError> {
        debug!("WebDAV upload to {}", self.file_url);

        self.ensure_dir().await?;

        let body = tokio::fs::read(src).await?;
        let response = self
            .client
            .put(&self.file_url)
            .header(AUTHORIZATION, &self.auth_header)
            .header(CONTENT_TYPE, "application/json")
            .body(body)
            .send()
            .await?;

        match response.status() {
            s if s.is_success() => {
                debug!("WebDAV upload complete");
                Ok(())
            }
            other => Err(Self::map_status(other)),
        }
    }

    #[instrument(skip(self))]
    async fn download(&self, dest: &Path) -> Result<(), BackupError> {
        debug!("WebDAV download from {}", self.file_url);

        let response = self
            .client
            .get(&self.file_url)
            .header(AUTHORIZATION, &self.auth_header)
            .send()
            .await?;

        match response.status() {
            s if s.is_success() => {
                let bytes = response.bytes().await?;
                tokio::fs::write(dest, &bytes).await?;
                debug!("WebDAV download complete ({} bytes)", bytes.len());
                Ok(())
            }
            other => Err(Self::map_status(other)),
        }
    }
}
