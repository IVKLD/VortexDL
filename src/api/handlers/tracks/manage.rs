use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::api::{
    errors::{ApiError, ErrorCode},
    state::AppState,
};

#[derive(Deserialize, ToSchema)]
pub struct DeleteTracksPayload {
    pub ids: Vec<i64>,
}

#[utoipa::path(
    method(delete),
    path = "/api/downloads/{id}",
    params(
        ("id" = i64, Path, description = "Track ID to remove")
    ),
    responses(
        (status = 200, description = "Track deleted successfully")
    )
)]
pub async fn remove_track(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, ApiError> {
    let mut storage = state.storage.write().await;

    match storage.remove_track(id).await {
        Ok(Some(_)) => Ok(()),
        Ok(None) => Err(ApiError::not_found(format!("Track with ID {id} not found"))
            .with_code(ErrorCode::TrackNotFound)),
        Err(e) => {
            Err(ApiError::internal(format!("Failed to delete file: {e}"))
                .with_code(ErrorCode::IoError))
        }
    }
}

#[utoipa::path(
    method(delete),
    path = "/api/downloads",
    request_body = DeleteTracksPayload,
    responses(
        (status = 200, description = "Tracks deleted successfully")
    )
)]
pub async fn remove_tracks(
    State(state): State<AppState>,
    Json(payload): Json<DeleteTracksPayload>,
) -> Result<impl IntoResponse, ApiError> {
    let mut storage = state.storage.write().await;

    storage
        .remove_tracks_batch(payload.ids)
        .await
        .map_err(|e| {
            ApiError::internal(format!("Failed to delete tracks: {e}"))
                .with_code(ErrorCode::IoError)
        })?;

    Ok(StatusCode::OK)
}
