use axum::{
    Json,
    extract::{Path, Query, State},
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use tokio_util::io::ReaderStream;

use crate::{
    api::{errors::{ApiError, ErrorCode}, state::AppState},
    storage::MusicStorage,
    types::api::{AudioFormat, TrackRecord},
};

#[derive(Deserialize)]
pub struct TracksQuery {
    pub sort: Option<String>,
    pub order: Option<String>,
    pub limit: Option<usize>,
}

pub async fn get_tracks(
    State(state): State<AppState>,
    Query(query): Query<TracksQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let storage = state.storage.read().await;

    let mut tracks = storage
        .tracks
        .iter()
        .map(|(id, data)| {
            let path = &data.path;
            let format = AudioFormat::from_path(path);

            TrackRecord {
                id: *id as u32,
                artist: data.artist.clone(),
                title: data.title.clone(),
                format,
                artwork_url: data.artwork_url.clone(),
                source_url: data.source_url.clone(),
                created_at: data.created_at,
                size: data.size,
                position: data.position.unwrap_or(u32::MAX),
            }
        })
        .collect::<Vec<_>>();

    let sort = query.sort.as_deref().unwrap_or("position");
    let order = query.order.as_deref().unwrap_or("asc");

    tracks.sort_by(|a, b| {
        let cmp = match sort {
            "name" => a.title.to_lowercase().cmp(&b.title.to_lowercase()),
            "date" => a.created_at.cmp(&b.created_at),
            _ => a.position.cmp(&b.position),
        };

        if order == "desc" { cmp.reverse() } else { cmp }
    });

    if let Some(limit) = query.limit {
        tracks.truncate(limit);
    }

    Ok(Json(tracks))
}

pub async fn indexing_tracks(State(state): State<AppState>) -> Result<impl IntoResponse, ApiError> {
    MusicStorage::run_background_indexing(state.storage.clone()).await;
    Ok(StatusCode::OK)
}

pub async fn remove_track(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, ApiError> {
    let mut storage = state.storage.write().await;

    storage
        .remove_track(id)
        .map_err(|e| {
            ApiError::internal(format!("Failed to delete file: {e}"))
                .with_code(ErrorCode::IoError)
        })?
        .ok_or_else(|| {
            ApiError::not_found(format!("Track with ID {id} not found"))
                .with_code(ErrorCode::TrackNotFound)
        })?;

    Ok(())
}

pub async fn stream_track(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Response, ApiError> {
    let path = state
        .storage
        .read()
        .await
        .tracks
        .get(&id)
        .map(|d| d.path.clone())
        .ok_or_else(|| ApiError::not_found(format!("Track with ID {id} not found")).with_code(ErrorCode::TrackNotFound))?;

    if !path.exists() {
        return Err(ApiError::not_found(format!("File not found: {:?}", path)).with_code(ErrorCode::FileNotFound));
    }

    let file = tokio::fs::File::open(&path)
        .await
        .map_err(|e| ApiError::internal(format!("Failed to open file: {e}")).with_code(ErrorCode::IoError))?;

    let metadata = file
        .metadata()
        .await
        .map_err(|e| ApiError::internal(format!("Failed to read metadata: {e}")).with_code(ErrorCode::IoError))?;

    let mime = mime_guess::from_path(&path).first_or_octet_stream();
    let body = axum::body::Body::from_stream(ReaderStream::new(file));

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, mime.to_string())
        .header(header::CONTENT_LENGTH, metadata.len().to_string())
        .header(header::ACCEPT_RANGES, "bytes")
        .body(body)
        .map_err(|e| ApiError::internal(format!("Failed to build response: {e}")).with_code(ErrorCode::InternalError))
}
