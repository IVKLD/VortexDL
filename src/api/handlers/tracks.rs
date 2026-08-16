use axum::{
    Json,
    extract::{Path, Query, Request, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use tower::ServiceExt;
use tower_http::services::ServeFile;
use utoipa::IntoParams;

use crate::{
    api::{
        errors::{ApiError, ErrorCode},
        state::AppState,
        types::MusicTrackRecord,
    },
    storage::MusicStorage,
};

#[derive(Deserialize, IntoParams)]
pub struct TracksQuery {
    pub sort: Option<String>,
    pub order: Option<String>,
    pub limit: Option<usize>,
}

#[utoipa::path(
    method(get),
    path = "/api/downloads",
    params(
        TracksQuery
    ),
    responses(
        (status = 200, description = "Get list of local track records", body = Vec<MusicTrackRecord>)
    )
)]
pub async fn get_tracks(
    State(state): State<AppState>,
    Query(query): Query<TracksQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let storage = state.storage.read().await;

    let mut tracks = storage
        .tracks
        .iter()
        .map(|(id, data)| MusicTrackRecord::from_local_track(*id, data))
        .collect::<Vec<_>>();

    let sort = query.sort.as_deref().unwrap_or("date");
    let order = query.order.as_deref().unwrap_or("desc");

    tracks.sort_by(|a, b| {
        let cmp = match sort {
            "name" => a.title.to_lowercase().cmp(&b.title.to_lowercase()),
            _ => b.created_at.cmp(&a.created_at),
        };

        let cmp = cmp.then_with(|| a.id.cmp(&b.id));

        if order == "desc" { cmp.reverse() } else { cmp }
    });

    if let Some(limit) = query.limit {
        tracks.truncate(limit);
    }

    Ok(Json(tracks))
}

#[utoipa::path(
    method(post),
    path = "/api/library/reindex",
    responses(
        (status = 200, description = "Reindexing triggered successfully")
    )
)]
pub async fn reindex_library(State(state): State<AppState>) -> Result<impl IntoResponse, ApiError> {
    let base_path = state.settings.output_path().await;
    let tracks = MusicStorage::scan_library(&base_path).await;
    state.storage.write().await.update_tracks(tracks);
    Ok(StatusCode::OK)
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

#[derive(Deserialize, utoipa::ToSchema)]
pub struct DeleteTracksPayload {
    pub ids: Vec<i64>,
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
