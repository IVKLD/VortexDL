use axum::{
    Json,
    extract::{Query, State},
    response::IntoResponse,
};
use soundcloud_rs::TracksQuery;
use url::Url;
use yt_audio_downloader::{YoutubeAudioDownloader, search_youtube};

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

fn youtube_meta_to_item(
    meta: &yt_audio_downloader::models::VideoMetadata,
) -> (i64, String, SearchTrackItem) {
    let id = yt_audio_downloader::youtube_id_to_i64(&meta.id);
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
    if trimmed.is_empty() {
        return Err(
            ApiError::bad_request("Search query cannot be empty").with_code(ErrorCode::EmptyUrl)
        );
    }

    let limit = params.limit.unwrap_or(20) as usize;
    let offset = params.offset.unwrap_or(0) as usize;
    let provider = params.provider.unwrap_or(SearchProviderParam::Soundcloud);
    let allow_yt = provider == SearchProviderParam::Youtube;
    let allow_sc = provider == SearchProviderParam::Soundcloud;

    let yt_duration_filter = params.duration.unwrap_or(SearchDurationFilter::Any);

    let yt_fut = async {
        if allow_yt && offset == 0 {
            if yt_audio_downloader::is_youtube_url(trimmed)
                && let Ok(meta) = YoutubeAudioDownloader::new().fetch_metadata(trimmed).await
            {
                return Some(vec![meta]);
            }
            let yt_fetch_count = if yt_duration_filter != SearchDurationFilter::Any {
                40
            } else {
                limit
            };
            search_youtube(trimmed, yt_fetch_count).await.ok()
        } else {
            None
        }
    };
    let sc_fut = async {
        if allow_sc {
            let sc_query = TracksQuery {
                q: Some(params.query.clone()),
                limit: params.limit,
                offset: params.offset,
                ..Default::default()
            };
            state.client.search_tracks(Some(&sc_query)).await.ok()
        } else {
            None
        }
    };

    let (yt_res, sc_res) = tokio::join!(yt_fut, sc_fut);

    let mut combined_tracks = Vec::new();
    let mut has_more = false;

    if let Some(yt_videos) = yt_res {
        let mut yt_cache = state.youtube_cache.write().await;
        for meta in yt_videos {
            let matches_duration = match yt_duration_filter {
                SearchDurationFilter::Short => meta.duration_seconds < 120,
                SearchDurationFilter::Medium => {
                    meta.duration_seconds >= 120 && meta.duration_seconds <= 600
                }
                SearchDurationFilter::Long => {
                    meta.duration_seconds > 600 && meta.duration_seconds <= 1800
                }
                SearchDurationFilter::Epic => meta.duration_seconds > 1800,
                SearchDurationFilter::Any => true,
            };
            if !matches_duration {
                continue;
            }
            let (id, vid, item) = youtube_meta_to_item(&meta);
            yt_cache.insert(id, vid);
            combined_tracks.push(item);
            if combined_tracks.len() >= limit {
                break;
            }
        }
    }

    if let Some(sc_results) = sc_res {
        has_more = sc_results.next_href.is_some();
        let sc_tracks = sc_results.collection.into_iter().filter_map(|track| {
            let id = track.id?;
            let artist = track
                .user
                .as_ref()
                .and_then(|u| u.username.clone())
                .unwrap_or_else(|| "Unknown".to_string());
            let artwork_url = track
                .artwork_url
                .and_then(|url| Url::parse(&url).ok())
                .map(|url| soundcloud::resize_artwork_url(url, "-t200x200").to_string());

            Some(SearchTrackItem {
                id,
                title: track.title.unwrap_or_default(),
                artist,
                artwork_url,
                duration: track.duration,
                playback_count: track.playback_count,
                permalink_url: track.permalink_url,
                genre: track.genre,
            })
        });
        combined_tracks.extend(sc_tracks);
    }

    Ok(Json(SearchResponse {
        tracks: combined_tracks,
        has_more,
    }))
}
