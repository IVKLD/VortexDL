use std::fs;

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};

use crate::{
    api::{
        errors::ApiError,
        models::{AudioFormat, TrackRecord},
        state::AppState,
    },
    storage::MusicStorage,
};

pub async fn get_tracks(State(state): State<AppState>) -> Result<impl IntoResponse, ApiError> {
    let storage = state.storage.read().await;

    let tracks = storage
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
            })
        })
        .collect::<Vec<_>>();

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
