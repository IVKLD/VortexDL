use std::path::PathBuf;

use serde::Deserialize;
use url::Url;
use utoipa::ToSchema;

use super::{
    backends::{HttpUrlBackend, LocalBackend, WebDavBackend},
    error::BackupError,
    service::BackupService,
};

#[derive(Debug, Clone, Copy, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub enum BackupAction {
    Export,
    Import,
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WebDavBackupConfig {
    #[schema(value_type = String)]
    pub base_url: Url,
    #[schema(value_type = String)]
    pub remote_dir: PathBuf,
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub enum BackupTarget {
    Local {
        #[schema(value_type = String)]
        path: PathBuf,
    },
    Url {
        #[schema(value_type = String)]
        url: Url,
    },
    WebDav(WebDavBackupConfig),
}

impl BackupTarget {
    pub async fn execute(&self, action: BackupAction) -> Result<(), BackupError> {
        match self {
            Self::Local { path } => {
                BackupService::new(LocalBackend::new(path))
                    .execute_action(action)
                    .await
            }
            Self::Url { url } => {
                let backend = HttpUrlBackend::new(url.as_str())?;
                BackupService::new(backend).execute_action(action).await
            }
            Self::WebDav(webdav) => {
                let backend = WebDavBackend::new(
                    webdav.base_url.as_str(),
                    webdav.remote_dir.to_string_lossy(),
                    &webdav.username,
                    &webdav.password,
                )?;
                BackupService::new(backend).execute_action(action).await
            }
        }
    }
}
