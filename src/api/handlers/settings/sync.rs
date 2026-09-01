use std::path::PathBuf;

use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use serde::{Deserialize, Serialize};
use url::Url;
use utoipa::ToSchema;

use crate::{
    api::{errors::ApiError, state::AppState},
    backup::{
        DataSnapshot, SyncError, SyncService,
        strategies::{LocalStrategy, UrlStrategy, WebDavStrategy},
    },
};

#[derive(Debug, Clone, Copy, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub enum SyncAction {
    Export,
    Import,
}

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub enum SyncProvider {
    Local {
        #[schema(value_type = String)]
        path: PathBuf,
    },
    Url {
        #[schema(value_type = String)]
        url: Url,
    },
    WebDav {
        #[schema(value_type = String)]
        base_url: Url,
        #[schema(value_type = String)]
        remote_dir: PathBuf,
        username: String,
        password: String,
    },
}

impl SyncProvider {
    pub async fn execute(&self, action: SyncAction) -> Result<(), SyncError> {
        match action {
            SyncAction::Export => self.export().await,
            SyncAction::Import => self.import().await,
        }
    }

    async fn export(&self) -> Result<(), SyncError> {
        match self {
            Self::Local { path } => SyncService::new(LocalStrategy::new(path)).export().await,
            Self::Url { url } => {
                let strategy = UrlStrategy::new(url.as_str())?;
                SyncService::new(strategy).export().await
            }
            Self::WebDav {
                base_url,
                remote_dir,
                username,
                password,
            } => {
                let strategy = WebDavStrategy::new(
                    base_url.as_str(),
                    remote_dir.to_string_lossy(),
                    username,
                    password,
                )?;
                SyncService::new(strategy).export().await
            }
        }
    }

    async fn import(&self) -> Result<(), SyncError> {
        match self {
            Self::Local { path } => SyncService::new(LocalStrategy::new(path)).import().await,
            Self::Url { url } => {
                let strategy = UrlStrategy::new(url.as_str())?;
                SyncService::new(strategy).import().await
            }
            Self::WebDav {
                base_url,
                remote_dir,
                username,
                password,
            } => {
                let strategy = WebDavStrategy::new(
                    base_url.as_str(),
                    remote_dir.to_string_lossy(),
                    username,
                    password,
                )?;
                SyncService::new(strategy).import().await
            }
        }
    }
}

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SyncRequest {
    pub provider: SyncProvider,
    pub action: SyncAction,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotResponse {
    pub sync_records_count: usize,
}

#[utoipa::path(
    method(get),
    path = "/api/settings/sync/snapshot",
    responses(
        (status = 200, description = "Get snapshot metadata", body = SnapshotResponse)
    )
)]
pub async fn get_snapshot_handler() -> Result<impl IntoResponse, ApiError> {
    let snapshot = DataSnapshot::from_database()?;
    let response = SnapshotResponse {
        sync_records_count: snapshot.synced_ids.len(),
    };
    Ok((StatusCode::OK, Json(response)))
}

#[utoipa::path(
    method(post),
    path = "/api/settings/sync",
    request_body = SyncRequest,
    responses(
        (status = 200, description = "Sync operation completed successfully")
    )
)]
pub async fn sync_handler(
    State(_state): State<AppState>,
    Json(payload): Json<SyncRequest>,
) -> Result<StatusCode, ApiError> {
    payload.provider.execute(payload.action).await?;
    Ok(StatusCode::OK)
}
