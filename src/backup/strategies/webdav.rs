use std::path::Path;

use reqwest::{
    Client, Method, StatusCode,
    header::{AUTHORIZATION, CONTENT_TYPE},
};
use tracing::{debug, instrument};

use crate::backup::{error::SyncError, strategy::ISyncStrategy};

const SNAPSHOT_FILENAME: &str = "state.json";

pub struct WebDavStrategy {
    client: Client,
    file_url: String,
    dir_url: String,
    auth_header: String,
}

impl WebDavStrategy {
    pub fn new(
        base_url: impl AsRef<str>,
        remote_dir: impl AsRef<str>,
        username: impl AsRef<str>,
        password: impl AsRef<str>,
    ) -> Result<Self, SyncError> {
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

    async fn ensure_dir(&self) -> Result<(), SyncError> {
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

    fn map_status(status: StatusCode) -> SyncError {
        match status {
            StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => SyncError::Auth,
            StatusCode::NOT_FOUND => SyncError::NotFound,
            other => SyncError::HttpStatus(other),
        }
    }
}

impl ISyncStrategy for WebDavStrategy {
    #[instrument(skip(self), fields(url = %self.file_url))]
    async fn upload(&self, src: &Path) -> Result<(), SyncError> {
        self.ensure_dir().await?;

        let body = tokio::fs::read(src).await?;
        debug!(bytes = body.len(), "WebDAV PUT");

        let response = self
            .client
            .put(&self.file_url)
            .header(AUTHORIZATION, &self.auth_header)
            .header(CONTENT_TYPE, "application/json")
            .body(body)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(Self::map_status(response.status()))
        }
    }

    #[instrument(skip(self), fields(url = %self.file_url))]
    async fn download(&self, dest: &Path) -> Result<(), SyncError> {
        let response = self
            .client
            .get(&self.file_url)
            .header(AUTHORIZATION, &self.auth_header)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(Self::map_status(response.status()));
        }

        let bytes = response.bytes().await?;
        tokio::fs::write(dest, &bytes).await?;

        debug!(bytes = bytes.len(), "WebDAV GET complete");
        Ok(())
    }
}
