use axum::{
    extract::{Path, Request, State},
    response::{IntoResponse, Response},
};
use tower::ServiceExt;
use tower_http::services::ServeFile;

use crate::api::{
    errors::{ApiError, ErrorCode},
    state::AppState,
};

#[utoipa::path(
    method(get),
    path = "/api/downloads/{id}/stream",
    params(
        ("id" = i64, Path, description = "Track ID to stream")
    ),
    responses(
        (status = 200, description = "Audio stream of track file"),
        (status = 206, description = "Partial content of track file")
    )
)]
pub async fn stream_track(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    req: Request,
) -> Result<Response, ApiError> {
    let path = state
        .storage
        .read()
        .await
        .tracks
        .get(&id)
        .map(|d| d.path.clone())
        .ok_or_else(|| {
            ApiError::not_found(format!("Track with ID {id} not found"))
                .with_code(ErrorCode::TrackNotFound)
        })?;

    if !path.exists() {
        return Err(ApiError::not_found(format!("File not found: {:?}", path))
            .with_code(ErrorCode::FileNotFound));
    }

    let service = ServeFile::new(path);
    let res = service.oneshot(req).await.map_err(|e| {
        ApiError::internal(format!("Failed to stream file: {e}")).with_code(ErrorCode::IoError)
    })?;

    Ok(res.into_response())
}
