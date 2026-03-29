use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use std::fs;

use super::{
    errors::ApiError,
    models::{
        DeleteResponse, DownloadLikesRequest, DownloadPlaylistRequest, DownloadQueuedResponse,
        DownloadedTrack, HealthResponse, ListResponse,
    },
    state::AppState,
};
use crate::{downloader, storage::MusicStorage};

// ── GET /health ───────────────────────────────────────────────────────────────

pub async fn health() -> impl IntoResponse {
    (StatusCode::OK, Json(HealthResponse { status: "ok" }))
}

// ── POST /download/playlist ───────────────────────────────────────────────────

pub async fn download_playlist(
    State(state): State<AppState>,
    Json(body): Json<DownloadPlaylistRequest>,
) -> Result<impl IntoResponse, ApiError> {
    if body.url.is_empty() {
        return Err(ApiError::bad_request("Field `url` must not be empty"));
    }

    let client = state.client.clone();
    let output_dir = state.output_dir.clone();
    let url = body.url.clone();

    tokio::spawn(async move {
        let mut storage = MusicStorage::new();
        storage.indexing(std::path::Path::new(output_dir.as_str()));

        if let Err(e) =
            downloader::download_playlist(&mut storage, &client, &url, &output_dir).await
        {
            tracing::error!("Playlist download failed: {e}");
        }
    });

    Ok((
        StatusCode::ACCEPTED,
        Json(DownloadQueuedResponse {
            status: "queued",
            message: format!("Playlist download started: {}", body.url),
        }),
    ))
}

// ── POST /download/likes ──────────────────────────────────────────────────────

pub async fn download_likes(
    State(state): State<AppState>,
    Json(body): Json<DownloadLikesRequest>,
) -> Result<impl IntoResponse, ApiError> {
    if body.url.is_empty() {
        return Err(ApiError::bad_request("Field `url` must not be empty"));
    }

    let client = state.client.clone();
    let config = state.config.clone();
    let output_dir = state.output_dir.clone();
    let url = body.url.clone();

    tokio::spawn(async move {
        let storage = MusicStorage::new();

        if let Err(e) =
            downloader::download_likes(&storage, &client, &url, &output_dir, &config).await
        {
            tracing::error!("Likes download failed: {e}");
        }
    });

    Ok((
        StatusCode::ACCEPTED,
        Json(DownloadQueuedResponse {
            status: "queued",
            message: format!("Likes download started: {}", body.url),
        }),
    ))
}

// ── GET /downloads ────────────────────────────────────────────────────────────

pub async fn list_downloads(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, ApiError> {
    let output_dir = state.output_dir.as_str();

    let tracks = fs::read_dir(output_dir)
        .map_err(|e| ApiError::internal(format!("Cannot read output dir: {e}")))?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.is_file()
                && path.extension().and_then(|e| e.to_str()) == Some("mp3")
            {
                Some(DownloadedTrack {
                    filename: path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .into_owned(),
                })
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    Ok(Json(ListResponse { tracks }))
}

// ── DELETE /downloads/:filename ───────────────────────────────────────────────

pub async fn delete_download(
    State(state): State<AppState>,
    Path(filename): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    // Guard against path traversal.
    if filename.contains('/') || filename.contains('\\') || filename.contains("..") {
        return Err(ApiError::bad_request("Invalid filename"));
    }

    let file_path = format!("{}/{}", state.output_dir, filename);

    if !std::path::Path::new(&file_path).exists() {
        return Err(ApiError::not_found(format!("File not found: {filename}")));
    }

    fs::remove_file(&file_path)
        .map_err(|e| ApiError::internal(format!("Failed to delete file: {e}")))?;

    Ok(Json(DeleteResponse { deleted: true, filename }))
}
