use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use serde::Serialize;
use utoipa::ToSchema;

use crate::{
    api::errors::ApiError,
    app::AppState,
    infra::db::{DeletedTrackRecord, clear_deleted_history, get_stats},
};

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct LibraryStatsResponse {
    pub total_tracks: usize,
    pub total_size: u64,
    pub total_downloaded: u64,
    pub total_downloaded_bytes: u64,
    pub total_deleted: u64,
    pub total_deleted_bytes: u64,
    pub recent_deleted: Vec<DeletedTrackRecord>,
}

#[utoipa::path(
    method(get),
    path = "/api/downloads/stats",
    responses(
        (status = 200, description = "Get library and download statistics", body = LibraryStatsResponse)
    )
)]
pub async fn get_library_stats(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, ApiError> {
    let storage = state.library.read().await;
    let total_tracks = storage.values().filter(|t| !t.is_archived).count();
    let total_size: u64 = storage
        .values()
        .filter(|t| !t.is_archived)
        .map(|t| t.size)
        .sum();

    let persistent = get_stats().unwrap_or_default();

    let total_downloaded = persistent
        .total_downloaded
        .max(total_tracks as u64 + persistent.total_deleted);
    let total_downloaded_bytes = persistent
        .total_downloaded_bytes
        .max(total_size + persistent.total_deleted_bytes);

    Ok(Json(LibraryStatsResponse {
        total_tracks,
        total_size,
        total_downloaded,
        total_downloaded_bytes,
        total_deleted: persistent.total_deleted,
        total_deleted_bytes: persistent.total_deleted_bytes,
        recent_deleted: persistent.deleted_history,
    }))
}

#[utoipa::path(
    method(delete),
    path = "/api/downloads/stats",
    responses(
        (status = 200, description = "Cleared deleted tracks history")
    )
)]
pub async fn clear_deleted_stats() -> Result<impl IntoResponse, ApiError> {
    clear_deleted_history().map_err(|e| ApiError::internal(e.to_string()))?;
    Ok(StatusCode::OK)
}
