use std::time::{Duration, Instant};

use axum::{
    Json,
    extract::{Path, Query, State},
    response::IntoResponse,
};
use url::Url;

use super::types::{StreamQuery, StreamUrlResponse};
use crate::{
    api::{
        errors::{ApiError, ErrorCode},
        state::AppState,
    },
    utils::{proxy::race_proxies, soundcloud},
};

pub async fn resolve_stream_url_internal(
    state: &AppState,
    id: i64,
    url_param: Option<String>,
) -> Result<String, ApiError> {
    if let Some((cached_url, instant)) = state.stream_cache.read().await.get(&id)
        && instant.elapsed() < Duration::from_secs(3 * 3600)
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
                        let client =
                            yt_audio_downloader::create_http_client_with_proxy(Some(&proxy));
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
            .insert(id, (stream_url.clone(), Instant::now()));
        return Ok(stream_url);
    }

    if let Ok((url, _)) = soundcloud::resolve_stream_url(&state.client, id).await {
        state
            .stream_cache
            .write()
            .await
            .insert(id, (url.clone(), Instant::now()));
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
        .insert(id, (stream_url.clone(), Instant::now()));
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
