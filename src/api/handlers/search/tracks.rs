use axum::{
    Json,
    extract::{Query, State},
    response::IntoResponse,
};
use soundcloud_rs::TracksQuery;
use url::Url;
use yt_audio_downloader::{YoutubeAudioDownloader, search_youtube_page_with_client};

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

    let query = (!trimmed.is_empty())
        .then_some(trimmed)
        .ok_or_else(|| ApiError::bad_request("Search query cannot be empty").with_code(ErrorCode::EmptyUrl))?;

    let limit = params.limit.unwrap_or(20) as usize;
    let offset = params.offset.unwrap_or(0) as usize;
    let provider = params.provider.unwrap_or(SearchProviderParam::Soundcloud);
    let allow_yt = provider == SearchProviderParam::Youtube;
    let allow_sc = provider == SearchProviderParam::Soundcloud;
    let yt_duration_filter = params.duration.unwrap_or(SearchDurationFilter::Any);

    let client = state.http_client().await;

    let yt_fut = async {
        if !allow_yt {
            return None;
        }

        if offset == 0
            && yt_audio_downloader::is_youtube_url(query)
            && let Ok(meta) = YoutubeAudioDownloader::new()
                .client(client.clone())
                .fetch_metadata(query)
                .await
        {
            return Some((vec![meta], false));
        }

        let continuation = if offset > 0 {
            state.cache.get_continuation(query).await
        } else {
            None
        };

        let (videos, next_token) =
            search_youtube_page_with_client(query, continuation.as_deref(), client)
                .await
                .ok()?;

        let has_more = next_token.is_some();
        state.cache.set_continuation(query, next_token).await;

        Some((videos, has_more))
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

    if let Some((yt_videos, yt_has_more)) = yt_res {
        has_more = yt_has_more;
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
            state.cache.insert_youtube_id(id, vid).await;
            combined_tracks.push(item);
            if combined_tracks.len() >= limit {
                break;
            }
        }
    }

    if let Some(sc_results) = sc_res {
        has_more = sc_results.next_href.is_some();
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

            combined_tracks.push(SearchTrackItem {
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
    }

    Ok(Json(SearchResponse {
        tracks: combined_tracks,
        has_more,
    }))
}
