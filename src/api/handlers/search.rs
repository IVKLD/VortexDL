use axum::{
    Json,
    extract::{Path, Query, State},
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use soundcloud_rs::{Identifier, StreamType, TracksQuery};
use url::Url;
use utoipa::ToSchema;

use crate::{
    api::{
        errors::{ApiError, ErrorCode},
        state::AppState,
    },
    utils::soundcloud::resize_artwork_url,
};

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub query: String,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SearchTrackItem {
    pub id: i64,
    pub title: String,
    pub artist: String,
    pub artwork_url: Option<String>,
    pub duration: Option<i64>,
    pub playback_count: Option<i64>,
    pub permalink_url: Option<String>,
    pub genre: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SearchResponse {
    pub tracks: Vec<SearchTrackItem>,
    pub has_more: bool,
}

#[utoipa::path(
    method(get),
    path = "/api/search/tracks",
    params(
        ("query" = String, Query, description = "Search query"),
        ("limit" = Option<i32>, Query, description = "Max results to return"),
        ("offset" = Option<i32>, Query, description = "Result offset for pagination")
    ),
    responses(
        (status = 200, description = "Search results", body = SearchResponse)
    )
)]
pub async fn search_tracks(
    State(state): State<AppState>,
    Query(params): Query<SearchQuery>,
) -> Result<impl IntoResponse, ApiError> {
    if params.query.trim().is_empty() {
        return Err(
            ApiError::bad_request("Search query cannot be empty").with_code(ErrorCode::EmptyUrl)
        );
    }

    let query = TracksQuery {
        q: Some(params.query),
        limit: params.limit,
        offset: params.offset,
        ..Default::default()
    };

    let results = state
        .client
        .search_tracks(Some(&query))
        .await
        .map_err(|e| {
            ApiError::bad_request(format!("SoundCloud search failed: {e}"))
                .with_code(ErrorCode::SoundCloudError)
        })?;

    let has_more = results.next_href.is_some();

    let tracks: Vec<SearchTrackItem> = results
        .collection
        .into_iter()
        .filter_map(|track| {
            let id = track.id?;
            let title = track.title.unwrap_or_default();
            let artist = track
                .user
                .as_ref()
                .and_then(|u| u.username.clone())
                .unwrap_or_else(|| "Unknown".to_string());

            let artwork_url = track
                .artwork_url
                .and_then(|url| Url::parse(&url).ok())
                .map(|url| resize_artwork_url(url, "-t200x200").to_string());

            Some(SearchTrackItem {
                id,
                title,
                artist,
                artwork_url,
                duration: track.duration,
                playback_count: track.playback_count,
                permalink_url: track.permalink_url,
                genre: track.genre,
            })
        })
        .collect();

    Ok(Json(SearchResponse { tracks, has_more }))
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct StreamUrlResponse {
    pub url: String,
}

#[utoipa::path(
    method(get),
    path = "/api/search/tracks/{id}/stream",
    params(
        ("id" = i64, Path, description = "SoundCloud track ID")
    ),
    responses(
        (status = 200, description = "Resolved stream URL", body = StreamUrlResponse)
    )
)]
pub async fn get_stream_url(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, ApiError> {
    let stream_url = state
        .client
        .get_stream_url(&Identifier::Id(id), Some(&StreamType::Progressive))
        .await
        .map_err(|e| {
            ApiError::bad_request(format!("Failed to resolve stream: {e}"))
                .with_code(ErrorCode::SoundCloudError)
        })?;

    Ok(Json(StreamUrlResponse { url: stream_url }))
}
