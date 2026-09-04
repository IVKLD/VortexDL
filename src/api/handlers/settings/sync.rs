use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    api::{errors::ApiError, state::AppState},
    backup::{BackupAction, BackupProvider, BackupSnapshot},
};

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SyncRequest {
    pub provider: BackupProvider,
    pub action: BackupAction,
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
    let snapshot = BackupSnapshot::from_database()?;
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
