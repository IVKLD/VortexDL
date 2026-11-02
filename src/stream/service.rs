use axum::{
    body::Body,
    extract::Request,
    http::{HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};
use reqwest::{
    Client, Response as ReqwestResponse,
    header::{ACCEPT_RANGES, CONTENT_LENGTH, CONTENT_RANGE, CONTENT_TYPE, RANGE, USER_AGENT},
};
use tower::ServiceExt;
use tower_http::services::ServeFile;
use yt_audio_downloader::select_user_agent_for_url;

use crate::{
    api::errors::ApiError,
    domain::SourcePlatform,
    infra::net::HttpClientPool,
    library::LibraryService,
    providers::{HlsPlaylist, ProviderError, SoundCloudProvider, YouTubeProvider},
    search::cache::SearchCache,
    stream::{
        cache::{CachedStream, StreamCache},
        hls::{HlsStreamOutcome, HlsStreamer},
    },
};

#[derive(Clone)]
pub struct StreamService {
    library: LibraryService,
    cache: StreamCache,
    search_cache: SearchCache,
    http_clients: HttpClientPool,
    soundcloud: SoundCloudProvider,
    youtube: YouTubeProvider,
}

impl StreamService {
    pub fn new(
        library: LibraryService,
        cache: StreamCache,
        search_cache: SearchCache,
        http_clients: HttpClientPool,
        soundcloud: SoundCloudProvider,
        youtube: YouTubeProvider,
    ) -> Self {
        Self {
            library,
            cache,
            search_cache,
            http_clients,
            soundcloud,
            youtube,
        }
    }

    pub async fn serve(
        &self,
        id: i64,
        url_param: Option<String>,
        req: Request,
    ) -> Result<Response, ApiError> {
        if let Some(track) = self.library.get_track(id).await
            && track.path.exists()
        {
            let res = ServeFile::new(track.path).oneshot(req).await?;
            return Ok(res.into_response());
        }

        let range_header = req.headers().get(header::RANGE).cloned();
        let response = self.proxy_remote(id, url_param, range_header).await?;
        Ok(response)
    }

    pub async fn proxy_remote(
        &self,
        id: i64,
        url_param: Option<String>,
        range_header: Option<HeaderValue>,
    ) -> Result<Response, ProviderError> {
        let mut cached = self.resolve_stream(id, url_param.clone()).await?;

        if let Some(ref playlist) = cached.hls_playlist {
            let client = self.http_clients.get_or_create(cached.proxy.clone());
            match HlsStreamer::new(client, playlist.clone())
                .stream(range_header.as_ref())
                .await?
            {
                HlsStreamOutcome::Response(res) => return Ok(res),
                HlsStreamOutcome::Expired => {
                    self.cache.remove(id);
                    cached = self.resolve_stream(id, url_param.clone()).await?;
                }
            }
        }

        let mut active_client = self.http_clients.get_or_create(cached.proxy.clone());
        let mut active_url = cached.url.clone();
        let mut upstream_range = if HlsPlaylist::is_hls_url(&active_url) {
            None
        } else {
            range_header.as_ref()
        };

        let mut upstream_res =
            Self::send_upstream_request(&active_client, &active_url, upstream_range).await?;

        let status = upstream_res.status().as_u16();
        if matches!(status, 401 | 403 | 404 | 410) {
            self.cache.remove(id);
            if let Ok(fresh) = self.resolve_stream(id, url_param).await {
                let fresh_client = self.http_clients.get_or_create(fresh.proxy);
                let fresh_range = if HlsPlaylist::is_hls_url(&fresh.url) {
                    None
                } else {
                    range_header.as_ref()
                };
                if let Ok(res) =
                    Self::send_upstream_request(&fresh_client, &fresh.url, fresh_range).await
                    && (res.status().is_success() || res.status().as_u16() == 206)
                {
                    active_client = fresh_client;
                    active_url = fresh.url;
                    upstream_range = fresh_range;
                    upstream_res = res;
                }
            }
        }

        if HlsPlaylist::is_hls_response(&upstream_res, &active_url) {
            let is_full_body = upstream_range.is_none()
                || upstream_range
                    .and_then(|h| h.to_str().ok())
                    .is_some_and(|s| s.trim() == "bytes=0-");
            if !is_full_body {
                upstream_res =
                    Self::send_upstream_request(&active_client, &active_url, None).await?;
            }

            let playlist =
                HlsStreamer::resolve_playlist(&active_client, upstream_res, &active_url).await?;
            self.cache.set_hls_playlist(id, playlist.clone());

            return match HlsStreamer::new(active_client, playlist)
                .stream(range_header.as_ref())
                .await?
            {
                HlsStreamOutcome::Response(res) => Ok(res),
                HlsStreamOutcome::Expired => {
                    self.cache.remove(id);
                    Err(ProviderError::InvalidStream(
                        "Upstream HLS stream expired immediately after resolution".to_string(),
                    ))
                }
            };
        }

        Self::build_passthrough_response(upstream_res)
    }

    pub async fn resolve_stream(
        &self,
        id: i64,
        url_param: Option<String>,
    ) -> Result<CachedStream, ProviderError> {
        if let Some(cached) = self.cache.get(id) {
            return Ok(cached);
        }

        let target_url = if let Some(u) = url_param {
            Some(u)
        } else if let Some(vid) = self.search_cache.get_youtube_id(id) {
            Some(format!("https://www.youtube.com/watch?v={vid}"))
        } else {
            self.library
                .get_track(id)
                .await
                .and_then(|t| t.metadata.source_url)
        };

        let resolved = match target_url
            .as_deref()
            .map(SourcePlatform::detect_from_url_str)
        {
            Some(SourcePlatform::YouTube) => {
                let u = target_url.as_deref().unwrap();
                self.youtube.resolve_stream(u).await?
            }
            _ => {
                let cached_track = self.search_cache.get_soundcloud_track(id);
                self.soundcloud
                    .resolve_stream(cached_track.as_ref(), id)
                    .await?
            }
        };

        self.cache
            .insert(id, resolved.stream_url.clone(), resolved.proxy_used.clone());

        self.cache
            .get(id)
            .ok_or_else(|| ProviderError::InvalidStream("Failed to cache stream".to_string()))
    }

    pub async fn send_upstream_request(
        client: &Client,
        url: &str,
        range_header: Option<&HeaderValue>,
    ) -> Result<ReqwestResponse, ProviderError> {
        let mut req = client
            .get(url)
            .header(USER_AGENT, select_user_agent_for_url(url));

        if let Some(range) = range_header
            && let Ok(range_str) = range.to_str()
        {
            req = req.header(RANGE, range_str);
        }

        Ok(req.send().await?)
    }

    fn build_passthrough_response(
        upstream_res: ReqwestResponse,
    ) -> Result<Response, ProviderError> {
        let status = StatusCode::from_u16(upstream_res.status().as_u16()).unwrap_or(StatusCode::OK);
        let mut response_builder = Response::builder().status(status);

        for (name, value) in upstream_res.headers() {
            if matches!(
                name,
                &CONTENT_TYPE | &CONTENT_LENGTH | &CONTENT_RANGE | &ACCEPT_RANGES
            ) {
                response_builder = response_builder.header(name.as_str(), value.as_bytes());
            }
        }

        if upstream_res.headers().get(ACCEPT_RANGES).is_none() {
            response_builder = response_builder.header(ACCEPT_RANGES, "bytes");
        }

        let body = Body::from_stream(upstream_res.bytes_stream());
        response_builder
            .body(body)
            .map_err(|e| ProviderError::InvalidStream(e.to_string()))
    }
}
