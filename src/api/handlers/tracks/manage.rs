use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};

use crate::api::{
    errors::{ApiError, ErrorCode},
    state::AppState,
};

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
    match storage.remove_track(id).await? {
        Some(_) => Ok(StatusCode::OK),
        None => Err(ApiError::not_found(format!("Track with ID {id} not found"))
            .with_code(ErrorCode::TrackNotFound)),
    }
}

#[utoipa::path(
    method(delete),
    path = "/api/downloads",
    request_body = Vec<i64>,
    responses(
        (status = 200, description = "Tracks deleted successfully")
    )
)]
pub async fn remove_tracks(
    State(state): State<AppState>,
    Json(ids): Json<Vec<i64>>,
) -> Result<impl IntoResponse, ApiError> {
    let mut storage = state.storage.write().await;
    storage.remove_tracks_batch(ids).await?;
    Ok(StatusCode::OK)
}
