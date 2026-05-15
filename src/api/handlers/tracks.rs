use std::fs;

use axum::{
    Json,
    extract::{Path, Query, State},
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use tokio_util::io::ReaderStream;

use crate::{
    api::{
        errors::ApiError,
        models::{AudioFormat, TrackRecord},
        state::AppState,
    },
    storage::MusicStorage,
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
        .filter_map(|(id, data)| {
            let path = &data.path;
            let metadata = path.metadata().ok()?;
            let extension_str = path.extension()?.to_string_lossy().to_string();
            let format = match extension_str.to_lowercase().as_str() {
                "mp3" => AudioFormat::Mp3,
                "flac" => AudioFormat::Flac,
                "wav" => AudioFormat::Wav,
                _ => AudioFormat::Unknown,
            };

            Some(TrackRecord {
                id: *id as u32,
                artist: data.artist.clone(),
                title: data.title.clone(),
                format,
                artwork_url: data.artwork_url.clone(),
                source_url: data.source_url.clone(),
                created_at: metadata
                    .created()
                    .map(|t| {
                        t.duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs()
                    })
                    .unwrap_or(0),
                size: metadata.len(),
                position: data.position.unwrap_or(u32::MAX),
            })
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
    let path = {
        let storage = state.storage.read().await;
        storage.tracks.get(&id).cloned()
    };

    if let Some(data) = path {
        if data.path.exists() {
            fs::remove_file(&data.path)
                .map_err(|e| ApiError::internal(format!("Failed to delete file: {e}")))?;
        }

        let mut storage = state.storage.write().await;
        storage.remove_track(id);

        return Ok(());
    }

    Err(ApiError::not_found(format!(
        "Track with ID {} not found",
        id
    )))
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
        .ok_or_else(|| ApiError::not_found(format!("Track with ID {id} not found")))?;

    if !path.exists() {
        return Err(ApiError::not_found(format!("File not found: {:?}", path)));
    }

    let file = tokio::fs::File::open(&path)
        .await
        .map_err(|e| ApiError::internal(format!("Failed to open file: {e}")))?;

    let metadata = file
        .metadata()
        .await
        .map_err(|e| ApiError::internal(format!("Failed to read metadata: {e}")))?;

    let mime = mime_guess::from_path(&path).first_or_octet_stream();
    let body = axum::body::Body::from_stream(ReaderStream::new(file));

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, mime.to_string())
        .header(header::CONTENT_LENGTH, metadata.len().to_string())
        .header(header::ACCEPT_RANGES, "bytes")
        .body(body)
        .map_err(|e| ApiError::internal(format!("Failed to build response: {e}")))
}
