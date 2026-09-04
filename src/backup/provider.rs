use std::path::PathBuf;

use serde::Deserialize;
use url::Url;
use utoipa::ToSchema;

use crate::backup::{
    error::BackupError,
    service::BackupService,
    strategies::{LocalStrategy, UrlStrategy, WebDavStrategy},
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
pub enum BackupProvider {
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

impl BackupProvider {
    pub async fn execute(&self, action: BackupAction) -> Result<(), BackupError> {
        match action {
            BackupAction::Export => self.export().await,
            BackupAction::Import => self.import().await,
        }
    }

    async fn export(&self) -> Result<(), BackupError> {
        match self {
            Self::Local { path } => BackupService::new(LocalStrategy::new(path)).export().await,
            Self::Url { url } => {
                let strategy = UrlStrategy::new(url.as_str())?;
                BackupService::new(strategy).export().await
            }
            Self::WebDav(webdav) => {
                let strategy = WebDavStrategy::new(
                    webdav.base_url.as_str(),
                    webdav.remote_dir.to_string_lossy(),
                    &webdav.username,
                    &webdav.password,
                )?;
                BackupService::new(strategy).export().await
            }
        }
    }

    async fn import(&self) -> Result<(), BackupError> {
        match self {
            Self::Local { path } => BackupService::new(LocalStrategy::new(path)).import().await,
            Self::Url { url } => {
                let strategy = UrlStrategy::new(url.as_str())?;
                BackupService::new(strategy).import().await
            }
            Self::WebDav(webdav) => {
                let strategy = WebDavStrategy::new(
                    webdav.base_url.as_str(),
                    webdav.remote_dir.to_string_lossy(),
                    &webdav.username,
                    &webdav.password,
                )?;
                BackupService::new(strategy).import().await
            }
        }
    }
}
