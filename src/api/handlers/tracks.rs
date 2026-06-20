use std::io::ErrorKind;

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
        errors::{ApiError, ErrorCode},
        state::AppState,
        types::TrackRecord,
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
        .map(|(id, data)| TrackRecord::from_local_track(*id, data))
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
    MusicStorage::index_library(state.storage.clone()).await;
    Ok(StatusCode::OK)
}

pub async fn remove_track(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, ApiError> {
    let mut storage = state.storage.write().await;

    storage
        .remove_track(id)
        .await
        .map_err(|e| {
            ApiError::internal(format!("Failed to delete file: {e}")).with_code(ErrorCode::IoError)
        })?
        .ok_or_else(|| {
            ApiError::not_found(format!("Track with ID {id} not found"))
                .with_code(ErrorCode::TrackNotFound)
        })?;

    Ok(())
}

use axum::http::HeaderMap;
use tokio::io::{AsyncReadExt, AsyncSeekExt};

fn parse_range(range_header: &str, file_size: u64) -> Option<(u64, u64)> {
    if !range_header.starts_with("bytes=") {
        return None;
    }
    let range_val = &range_header["bytes=".len()..];
    let mut parts = range_val.split('-');
    let start_str = parts.next()?.trim();
    let end_str = parts.next()?.trim();

    if start_str.is_empty() {
        let suffix_len = end_str.parse::<u64>().ok()?;
        let start = file_size.saturating_sub(suffix_len);
        Some((start, file_size.saturating_sub(1)))
    } else {
        let start = start_str.parse::<u64>().ok()?;
        if end_str.is_empty() {
            Some((start, file_size.saturating_sub(1)))
        } else {
            let end = end_str.parse::<u64>().ok()?;
            Some((start, end.min(file_size.saturating_sub(1))))
        }
    }
}

pub async fn stream_track(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
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

    let mut file = tokio::fs::File::open(&path)
        .await
        .map_err(|e| match e.kind() {
            ErrorKind::NotFound => ApiError::not_found(format!("File not found: {:?}", path))
                .with_code(ErrorCode::FileNotFound),
            _ => ApiError::internal(format!("Failed to open file: {e}"))
                .with_code(ErrorCode::IoError),
        })?;

    let metadata = file.metadata().await.map_err(|e| {
        ApiError::internal(format!("Failed to read metadata: {e}")).with_code(ErrorCode::IoError)
    })?;

    let file_len = metadata.len();
    let mime = mime_guess::from_path(&path).first_or_octet_stream();

    let mut start = 0;
    let mut end = file_len.saturating_sub(1);
    let mut is_partial = false;

    if let Some((r_start, r_end)) = headers
        .get(header::RANGE)
        .and_then(|v| v.to_str().ok())
        .and_then(|h| parse_range(h, file_len))
        .filter(|&(r_start, r_end)| r_start <= r_end && r_start < file_len)
    {
        start = r_start;
        end = r_end;
        is_partial = true;
    }

    let length = end - start + 1;

    let body = if is_partial {
        if let Err(e) = file.seek(std::io::SeekFrom::Start(start)).await {
            return Err(ApiError::internal(format!("Failed to seek file: {e}"))
                .with_code(ErrorCode::IoError));
        }
        let reader = file.take(length);
        axum::body::Body::from_stream(ReaderStream::new(reader))
    } else {
        axum::body::Body::from_stream(ReaderStream::new(file))
    };

    let mut builder = Response::builder()
        .header(header::CONTENT_TYPE, mime.to_string())
        .header(header::ACCEPT_RANGES, "bytes");

    if is_partial {
        builder = builder
            .status(StatusCode::PARTIAL_CONTENT)
            .header(
                header::CONTENT_RANGE,
                format!("bytes {start}-{end}/{file_len}"),
            )
            .header(header::CONTENT_LENGTH, length.to_string());
    } else {
        builder = builder
            .status(StatusCode::OK)
            .header(header::CONTENT_LENGTH, file_len.to_string());
    }

    builder.body(body).map_err(|e| {
        ApiError::internal(format!("Failed to build response: {e}"))
            .with_code(ErrorCode::InternalError)
    })
}
