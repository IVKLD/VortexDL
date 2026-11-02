use std::path::PathBuf;

use serde::Deserialize;
use url::Url;
use utoipa::ToSchema;

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
