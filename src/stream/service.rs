use std::time::Instant;

use axum::{
    body::Body,
    extract::Request,
    http::{HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};
use reqwest::{
    Client,
    header::{ACCEPT_RANGES, CONTENT_LENGTH, CONTENT_RANGE, CONTENT_TYPE},
};
use tower::ServiceExt;
use tower_http::services::ServeFile;
use yt_audio_downloader::AudioStreamResponse;

use crate::{
    domain::SourcePlatform,
    infra::net::HttpClientPool,
    library::LibraryService,
    providers::{ByteRange, HlsPlaylist, SoundCloudProvider, YouTubeProvider},
    search::cache::SearchCache,
    stream::{
        cache::{CachedStream, StreamCache, StreamTarget},
        error::StreamError,
        hls::HlsStreamer,
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
    ) -> Result<Response, StreamError> {
        if let Some(track) = self.library.get_track(id).await
            && track.path.exists()
        {
            let res = ServeFile::new(track.path).oneshot(req).await?;
            return Ok(res.into_response());
        }

        let range_header = req.headers().get(header::RANGE);
        self.stream_remote(id, url_param.as_deref(), range_header).await
    }

    async fn stream_remote(
        &self,
        id: i64,
        url: Option<&str>,
        range: Option<&HeaderValue>,
    ) -> Result<Response, StreamError> {
        if let Ok(res) = self.dispatch(id, url, range).await {
            return Ok(res);
        }

        self.cache.remove(id);
        self.dispatch(id, url, range).await
    }

    async fn dispatch(
        &self,
        id: i64,
        url: Option<&str>,
        range: Option<&HeaderValue>,
    ) -> Result<Response, StreamError> {
        let cached = self.resolve_stream(id, url).await?;

        match cached.target {
            StreamTarget::YouTube(yt) => self.stream_youtube(&yt, range).await,
            StreamTarget::SoundCloud(playlist) => {
                let client = self.http_clients.get_or_create(cached.proxy.clone());
                self.stream_soundcloud(client, playlist, range).await
            }
        }
    }

    async fn stream_youtube(
        &self,
        info: &AudioStreamResponse,
        range_header: Option<&HeaderValue>,
    ) -> Result<Response, StreamError> {
        let total = info.content_length.unwrap_or(0);
        let range_str = range_header.and_then(|h| h.to_str().ok());
        let byte_range = if total > 0 {
            ByteRange::parse(range_str, total)
        } else {
            ByteRange::Full
        };

        let (start, end, status) = match byte_range {
            ByteRange::Partial(s, e) => (s, Some(e), StatusCode::PARTIAL_CONTENT),
            ByteRange::Unsatisfiable => {
                let res = Response::builder()
                    .status(StatusCode::RANGE_NOT_SATISFIABLE)
                    .header(CONTENT_RANGE, format!("bytes */{total}"))
                    .body(Body::empty())?;
                return Ok(res);
            }
            ByteRange::Full => (0, total.checked_sub(1), StatusCode::OK),
        };

        let mime = if info.mime_type.is_empty() {
            "audio/webm"
        } else {
            &info.mime_type
        };

        let stream = self.youtube.stream(info, start, end).await?;

        let mut builder = Response::builder()
            .status(status)
            .header(CONTENT_TYPE, mime)
            .header(ACCEPT_RANGES, "bytes");

        if let Some(end) = end {
            builder = builder.header(CONTENT_LENGTH, (end - start + 1).to_string());
            if total > 0 && status == StatusCode::PARTIAL_CONTENT {
                builder = builder.header(CONTENT_RANGE, format!("bytes {start}-{end}/{total}"));
            }
        }

        let res = builder.body(Body::from_stream(stream))?;
        Ok(res)
    }

    async fn stream_soundcloud(
        &self,
        client: Client,
        playlist: HlsPlaylist,
        range: Option<&HeaderValue>,
    ) -> Result<Response, StreamError> {
        let res = HlsStreamer::new(client, playlist).stream(range).await?;
        Ok(res)
    }

    async fn resolve_stream(
        &self,
        id: i64,
        url_param: Option<&str>,
    ) -> Result<CachedStream, StreamError> {
        if let Some(cached) = self.cache.get(id) {
            return Ok(cached);
        }

        let cached = match self.detect_platform(id, url_param).await {
            SourcePlatform::YouTube => self.resolve_youtube(id, url_param).await?,
            SourcePlatform::SoundCloud => self.resolve_soundcloud(id).await?,
            SourcePlatform::Local => {
                return Err(StreamError::UnsupportedPlatform(id));
            }
        };

        self.cache.insert(id, cached.clone());
        Ok(cached)
    }

    async fn detect_platform(&self, id: i64, url: Option<&str>) -> SourcePlatform {
        if let Some(url) = url {
            return SourcePlatform::detect_from_url_str(url);
        }
        if self.search_cache.get_youtube_id(id).is_some() {
            return SourcePlatform::YouTube;
        }
        if self.search_cache.get_soundcloud_track(id).is_some() {
            return SourcePlatform::SoundCloud;
        }
        if let Some(track) = self.library.get_track(id).await {
            return SourcePlatform::detect_from_optional_url(track.metadata.source_url.as_deref());
        }
        SourcePlatform::Local
    }

    async fn resolve_youtube(
        &self,
        id: i64,
        url_param: Option<&str>,
    ) -> Result<CachedStream, StreamError> {
        let url = if let Some(u) = url_param {
            u.to_string()
        } else if let Some(vid) = self.search_cache.get_youtube_id(id) {
            format!("https://www.youtube.com/watch?v={vid}")
        } else if let Some(track) = self.library.get_track(id).await
            && let Some(src) = track.metadata.source_url
        {
            src
        } else {
            return Err(StreamError::TrackNotFound(id));
        };

        let (info, proxy) = self.youtube.resolve_stream(&url).await?;
        Ok(CachedStream {
            target: StreamTarget::YouTube(Box::new(info)),
            proxy,
            instant: Instant::now(),
        })
    }

    async fn resolve_soundcloud(&self, id: i64) -> Result<CachedStream, StreamError> {
        let cached_track = self.search_cache.get_soundcloud_track(id);
        let stream = self
            .soundcloud
            .resolve_stream(cached_track.as_ref(), id)
            .await?;
        let client = self.http_clients.get_or_create(stream.proxy_url.clone());
        let playlist = HlsStreamer::fetch_playlist(&client, &stream.url).await?;

        Ok(CachedStream {
            target: StreamTarget::SoundCloud(playlist),
            proxy: stream.proxy_url,
            instant: Instant::now(),
        })
    }
}
