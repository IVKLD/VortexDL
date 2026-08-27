use axum::{
    Json,
    extract::{Path, Query, State},
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use soundcloud_rs::TracksQuery;
use url::Url;
use utoipa::ToSchema;
use yt_audio_downloader::{YoutubeAudioDownloader, search_youtube};

use crate::{
    api::{
        errors::{ApiError, ErrorCode},
        state::AppState,
    },
    utils::{filename, proxy::race_proxies, soundcloud},
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
    let trimmed = params.query.trim();
    if trimmed.is_empty() {
        return Err(
            ApiError::bad_request("Search query cannot be empty").with_code(ErrorCode::EmptyUrl)
        );
    }

    if let Ok(parsed_url) = Url::parse(trimmed)
        && yt_audio_downloader::is_youtube_url(parsed_url.as_str())
    {
        let downloader = YoutubeAudioDownloader::new();
        if let Ok(meta) = downloader.fetch_metadata(trimmed).await {
            let (id, vid, track_item) = youtube_meta_to_item(&meta);
            state.youtube_cache.write().await.insert(id, vid);
            return Ok(Json(SearchResponse {
                tracks: vec![track_item],
                has_more: false,
            }));
        }
    }

    let limit = params.limit.unwrap_or(20) as usize;
    let offset = params.offset.unwrap_or(0);

    let yt_fut = async {
        if offset == 0 {
            search_youtube(trimmed, limit).await.ok()
        } else {
            None
        }
    };
    let sc_query = TracksQuery {
        q: Some(params.query.clone()),
        limit: params.limit,
        offset: params.offset,
        ..Default::default()
    };
    let sc_fut = state.client.search_tracks(Some(&sc_query));

    let (yt_res, sc_res) = tokio::join!(yt_fut, sc_fut);

    let mut combined_tracks = Vec::new();
    let mut has_more = false;

    if let Some(yt_videos) = yt_res {
        let mut yt_cache = state.youtube_cache.write().await;
        for meta in yt_videos {
            let (id, vid, item) = youtube_meta_to_item(&meta);
            yt_cache.insert(id, vid);
            combined_tracks.push(item);
        }
    }

    if let Ok(sc_results) = sc_res {
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

#[derive(Debug, Deserialize)]
pub struct StreamQuery {
    pub url: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct StreamUrlResponse {
    pub url: String,
}

pub async fn resolve_stream_url_internal(
    state: &AppState,
    id: i64,
    url_param: Option<String>,
) -> Result<String, ApiError> {
    if let Some((cached_url, instant)) = state.stream_cache.read().await.get(&id)
        && instant.elapsed() < std::time::Duration::from_secs(3 * 3600)
    {
        return Ok(cached_url.clone());
    }

    let mut target_url = url_param;

    if target_url.is_none() {
        target_url = state
            .youtube_cache
            .read()
            .await
            .get(&id)
            .map(|vid| format!("https://www.youtube.com/watch?v={vid}"));
    }

    if target_url.is_none() {
        let storage = state.storage.read().await;
        if let Some(track) = storage.tracks.get(&id) {
            target_url = track.metadata.source_url.clone();
        }
    }

    if let Some(ref u) = target_url
        && let Ok(parsed_url) = Url::parse(u)
        && yt_audio_downloader::is_youtube_url(parsed_url.as_str())
        && let Ok(video_id) = yt_audio_downloader::extractor::extract_video_id(u)
    {
        let settings = state.settings.read().await.clone();
        let proxy_url = settings.network.get_proxy_url();
        let client = yt_audio_downloader::create_http_client_with_proxy(proxy_url);

        let direct_res = yt_audio_downloader::get_stream_info_with_client(
            &video_id,
            client,
            proxy_url.map(String::from),
        )
        .await;

        let stream_url = match direct_res {
            Ok(info) => info.stream_url,
            Err(direct_err) => {
                if !settings.network.use_proxy || settings.network.fallback_proxies.is_empty() {
                    return Err(ApiError::bad_request(format!(
                        "YouTube stream resolution failed: {direct_err}"
                    ))
                    .with_code(ErrorCode::NetworkError));
                }

                race_proxies(&settings, move |_, proxy| {
                    let v_id = video_id.clone();
                    async move {
                        let client = yt_audio_downloader::create_http_client_with_proxy(Some(&proxy));
                        let info = yt_audio_downloader::get_stream_info_with_client(
                            &v_id,
                            client,
                            Some(proxy),
                        )
                        .await?;
                        Ok(info.stream_url)
                    }
                })
                .await
                .map_err(|e| {
                    ApiError::bad_request(format!(
                        "YouTube stream resolution failed on all proxies: {e}"
                    ))
                    .with_code(ErrorCode::NetworkError)
                })?
            }
        };

        state
            .stream_cache
            .write()
            .await
            .insert(id, (stream_url.clone(), std::time::Instant::now()));
        return Ok(stream_url);
    }

    if let Ok((url, _)) = soundcloud::resolve_stream_url(&state.client, id).await {
        state
            .stream_cache
            .write()
            .await
            .insert(id, (url.clone(), std::time::Instant::now()));
        return Ok(url);
    }

    let settings = state.settings.read().await.clone();
    if !settings.network.use_proxy || settings.network.fallback_proxies.is_empty() {
        return Err(ApiError::bad_request(
            "Direct resolution failed and no fallback proxies configured",
        )
        .with_code(ErrorCode::SoundCloudError));
    }

    let stream_url = race_proxies(&settings, move |s, proxy| async move {
        let client = soundcloud::ClientBuilder::new(&s)
            .with_proxy(Some(&proxy))
            .build()
            .await?;
        let (url, _) = soundcloud::resolve_stream_url(&client, id).await?;
        Ok(url)
    })
    .await
    .map_err(|e| {
        ApiError::bad_request(format!("Proxy resolution failed: {e}"))
            .with_code(ErrorCode::SoundCloudError)
    })?;

    state
        .stream_cache
        .write()
        .await
        .insert(id, (stream_url.clone(), std::time::Instant::now()));
    Ok(stream_url)
}

#[utoipa::path(
    method(get),
    path = "/api/search/tracks/{id}/stream",
    params(
        ("id" = i64, Path, description = "Track ID"),
        ("url" = Option<String>, Query, description = "Track permalink or source URL")
    ),
    responses(
        (status = 200, description = "Resolved stream URL", body = StreamUrlResponse)
    )
)]
pub async fn get_stream_url(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Query(params): Query<StreamQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let url = resolve_stream_url_internal(&state, id, params.url).await?;
    Ok(Json(StreamUrlResponse { url }))
}
