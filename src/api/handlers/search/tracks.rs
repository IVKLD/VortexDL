use axum::{
    Json,
    extract::{Query, State},
    response::IntoResponse,
};
use soundcloud_rs::TracksQuery;
use url::Url;
use yt_audio_downloader::{
    VideoMetadata, YoutubeAudioDownloader, is_youtube_url, search_youtube_page_with_client,
    youtube_id_to_i64,
};

use super::types::{
    SearchDurationFilter, SearchProviderParam, SearchQuery, SearchResponse, SearchTrackItem,
};
use crate::{
    api::{
        errors::{ApiError, ErrorCode},
        state::AppState,
    },
    utils::{filename, soundcloud},
};

fn youtube_meta_to_item(meta: &VideoMetadata) -> (i64, String, SearchTrackItem) {
    let id = youtube_id_to_i64(&meta.id);
    let permalink_url = format!("https://www.youtube.com/watch?v={}", meta.id);
    let (artist, title) = filename::parse_track_metadata(&meta.title, &meta.author);
    let item = SearchTrackItem {
        id,
        title,
        artist,
        artwork_url: meta.thumbnail_url.clone(),
        duration: Some((meta.duration_seconds * 1000) as i64),
        playback_count: Some(meta.view_count as i64),
        permalink_url: Some(permalink_url),
        genre: None,
    };
    (id, meta.id.clone(), item)
}

fn matches_duration(duration_seconds: u64, filter: SearchDurationFilter) -> bool {
    match filter {
        SearchDurationFilter::Short => duration_seconds < 120,
        SearchDurationFilter::Medium => (120..=600).contains(&duration_seconds),
        SearchDurationFilter::Long => (601..=1800).contains(&duration_seconds),
        SearchDurationFilter::Epic => duration_seconds > 1800,
        SearchDurationFilter::Any => true,
    }
}

async fn search_youtube(
    state: &AppState,
    query: &str,
    limit: usize,
    offset: usize,
    duration: SearchDurationFilter,
) -> Result<SearchResponse, ApiError> {
    let client = state.http_client().await;

    if offset == 0
        && is_youtube_url(query)
        && let Ok(meta) = YoutubeAudioDownloader::new()
            .client(client.clone())
            .fetch_metadata(query)
            .await
    {
        let (id, vid, item) = youtube_meta_to_item(&meta);
        state.cache.insert_youtube_id(id, vid).await;
        return Ok(SearchResponse {
            tracks: vec![item],
            has_more: false,
        });
    }

    let continuation = if offset > 0 {
        state.cache.get_continuation(query).await
    } else {
        None
    };

    let Ok((videos, next_token)) =
        search_youtube_page_with_client(query, continuation.as_deref(), client).await
    else {
        return Ok(SearchResponse {
            tracks: Vec::new(),
            has_more: false,
        });
    };

    let has_more = next_token.is_some();
    state.cache.set_continuation(query, next_token).await;

    let mut tracks = Vec::new();
    for meta in videos {
        if !matches_duration(meta.duration_seconds, duration) {
            continue;
        }

        let (id, vid, item) = youtube_meta_to_item(&meta);
        state.cache.insert_youtube_id(id, vid).await;
        tracks.push(item);
        if tracks.len() >= limit {
            break;
        }
    }

    Ok(SearchResponse { tracks, has_more })
}

async fn search_soundcloud(
    state: &AppState,
    query: &str,
    limit: usize,
    offset: usize,
) -> Result<SearchResponse, ApiError> {
    let sc_query = TracksQuery {
        q: Some(query.to_string()),
        limit: Some(limit as i32),
        offset: Some(offset as i32),
        ..Default::default()
    };

    let Some(sc_results) = state.client.search_tracks(Some(&sc_query)).await.ok() else {
        return Ok(SearchResponse {
            tracks: Vec::new(),
            has_more: false,
        });
    };

    let has_more = sc_results.next_href.is_some();
    let mut tracks = Vec::new();

    for track in sc_results.collection {
        let Some(id) = track.id else { continue };
        state.cache.insert_soundcloud_track(id, track.clone()).await;
        let artist = track
            .user
            .as_ref()
            .and_then(|u| u.username.clone())
            .unwrap_or_else(|| "Unknown".to_string());
        let artwork_url = track
            .artwork_url
            .and_then(|url| Url::parse(&url).ok())
            .map(|url| soundcloud::resize_artwork_url(url, "-t200x200").to_string());

        tracks.push(SearchTrackItem {
            id,
            title: track.title.unwrap_or_default(),
            artist,
            artwork_url,
            duration: track.duration,
            playback_count: track.playback_count,
            permalink_url: track.permalink_url,
            genre: track.genre,
        });
    }

    Ok(SearchResponse { tracks, has_more })
}

#[utoipa::path(
    method(get),
    path = "/api/search/tracks",
    params(
        ("query" = String, Query, description = "Search query"),
        ("limit" = Option<i32>, Query, description = "Max results to return"),
        ("offset" = Option<i32>, Query, description = "Result offset for pagination"),
        ("provider" = Option<SearchProviderParam>, Query, description = "Search provider filter: youtube, soundcloud"),
        ("duration" = Option<SearchDurationFilter>, Query, description = "Duration filter: any, short, medium, long, epic")
    ),
    responses(
        (status = 200, description = "Search results", body = SearchResponse)
    )
)]
pub async fn search_tracks(
    State(state): State<AppState>,
    Query(params): Query<SearchQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let trimmed = params.query.trim();

    let query = (!trimmed.is_empty()).then_some(trimmed).ok_or_else(|| {
        ApiError::bad_request("Search query cannot be empty").with_code(ErrorCode::BadRequest)
    })?;

    let limit = (params.limit.unwrap_or(20).max(1) as usize).min(100);
    let offset = params.offset.unwrap_or(0).max(0) as usize;
    let provider = params.provider.unwrap_or(SearchProviderParam::Soundcloud);
    let duration = params.duration.unwrap_or(SearchDurationFilter::Any);

    let response = match provider {
        SearchProviderParam::Youtube => {
            search_youtube(&state, query, limit, offset, duration).await?
        }
        SearchProviderParam::Soundcloud => search_soundcloud(&state, query, limit, offset).await?,
    };

    Ok(Json(response))
}
