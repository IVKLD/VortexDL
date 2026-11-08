use std::pin::Pin;

use axum::{
    body::{Body, Bytes},
    http::{HeaderValue, StatusCode},
    response::Response,
};
use futures::{Stream, StreamExt};
use reqwest::{
    Client,
    header::{ACCEPT_RANGES, CONTENT_LENGTH, CONTENT_RANGE, CONTENT_TYPE, USER_AGENT},
};

use crate::{
    infra::net::DEFAULT_USER_AGENT,
    providers::{ByteRange, HlsPlaylist, HlsSegment, ProviderError},
};

type SegmentByteStream = Pin<Box<dyn Stream<Item = Result<Bytes, reqwest::Error>> + Send>>;

struct HlsStreamState {
    client: Client,
    segments: std::vec::IntoIter<HlsSegment>,
    current_stream: Option<SegmentByteStream>,
    current_offset: u64,
    req_range: Option<(u64, u64)>,
}

#[derive(Clone)]
pub struct HlsStreamer {
    client: Client,
    playlist: HlsPlaylist,
}

impl HlsStreamer {
    pub fn new(client: Client, playlist: HlsPlaylist) -> Self {
        Self { client, playlist }
    }

    async fn fetch_manifest(client: &Client, url: &str) -> Result<String, ProviderError> {
        let manifest = client
            .get(url)
            .header(USER_AGENT, DEFAULT_USER_AGENT)
            .send()
            .await?
            .text()
            .await?;
        Ok(manifest)
    }

    pub async fn fetch_playlist(client: &Client, url: &str) -> Result<HlsPlaylist, ProviderError> {
        let manifest = Self::fetch_manifest(client, url).await?;
        let mut playlist = HlsPlaylist::parse(&manifest, url)?;

        if let Some(nested_url) = playlist.single_nested_playlist_url() {
            let nested_url = nested_url.to_string();
            let nested_manifest = Self::fetch_manifest(client, &nested_url).await?;
            playlist = HlsPlaylist::parse(&nested_manifest, &nested_url)?;
        }

        Ok(playlist)
    }

    pub async fn stream(
        self,
        range_header: Option<&HeaderValue>,
    ) -> Result<Response, ProviderError> {
        let range_str = range_header.and_then(|h| h.to_str().ok());
        let (req_range, status) = match self.playlist.total_size {
            Some(ts) => match ByteRange::parse(range_str, ts) {
                ByteRange::Partial(start, end) => (Some((start, end)), StatusCode::PARTIAL_CONTENT),
                ByteRange::Unsatisfiable => {
                    return Ok(Response::builder()
                        .status(StatusCode::RANGE_NOT_SATISFIABLE)
                        .header(CONTENT_RANGE, format!("bytes */{ts}"))
                        .body(Body::empty())?);
                }
                ByteRange::Full => (None, StatusCode::OK),
            },
            None => (None, StatusCode::OK),
        };

        let segments = match req_range {
            Some((start, end)) => self.playlist.segments_in_range(start, end),
            None => self.playlist.segments,
        };

        let mut state = HlsStreamState {
            client: self.client,
            segments: segments.into_iter(),
            current_stream: None,
            current_offset: 0,
            req_range,
        };

        let first_chunk = match state.poll_first_chunk().await {
            Ok(Some(chunk)) => chunk,
            Ok(None) => Bytes::new(),
            Err(e)
                if e.status()
                    .is_some_and(|s| s.as_u16() == 403 || s.as_u16() == 401) =>
            {
                return Err(ProviderError::InvalidStream(
                    "HLS stream expired".to_string(),
                ));
            }
            Err(e) => return Err(e.into()),
        };

        let content_type = if first_chunk.get(4..8) == Some(b"ftyp") {
            "audio/mp4"
        } else {
            "audio/mpeg"
        };

        let stream = state.into_response_stream(first_chunk);
        let mut builder = Response::builder()
            .status(status)
            .header(CONTENT_TYPE, content_type)
            .header(ACCEPT_RANGES, "bytes");

        if let Some((start, end)) = req_range {
            let chunk_len = end - start + 1;
            builder = builder.header(CONTENT_LENGTH, chunk_len.to_string());
            if let Some(total_size) = self.playlist.total_size {
                builder =
                    builder.header(CONTENT_RANGE, format!("bytes {start}-{end}/{total_size}"));
            }
        } else if let Some(total_size) = self.playlist.total_size {
            builder = builder.header(CONTENT_LENGTH, total_size.to_string());
        }

        Ok(builder.body(Body::from_stream(stream))?)
    }
}

impl HlsStreamState {
    async fn poll_first_chunk(&mut self) -> Result<Option<Bytes>, reqwest::Error> {
        let Some(seg) = self.segments.next() else {
            return Ok(None);
        };

        let mut stream = fetch_segment_stream(&self.client, &seg).await?;
        let first = stream.next().await.transpose()?;
        self.current_stream = Some(stream);
        Ok(first)
    }

    fn into_response_stream(
        self,
        first_chunk: Bytes,
    ) -> Pin<Box<dyn Stream<Item = Result<Bytes, reqwest::Error>> + Send>> {
        let initial = (!first_chunk.is_empty()).then_some(first_chunk);

        Box::pin(futures::stream::unfold(
            (self, initial),
            |(mut state, mut buffered)| async move {
                loop {
                    if let Some(buf) = buffered.take()
                        && let Some(chunk) = state.slice_chunk(buf)
                    {
                        return Some((Ok(chunk), (state, None)));
                    }

                    if let Some(ref mut stream) = state.current_stream {
                        match stream.next().await {
                            Some(Ok(bytes)) => {
                                if let Some(chunk) = state.slice_chunk(bytes) {
                                    return Some((Ok(chunk), (state, None)));
                                }
                            }
                            Some(Err(e)) => return Some((Err(e), (state, None))),
                            None => {
                                state.current_stream = None;
                            }
                        }
                    }

                    let next_seg = state.segments.next()?;

                    match fetch_segment_stream(&state.client, &next_seg).await {
                        Ok(stream) => state.current_stream = Some(stream),
                        Err(e) => return Some((Err(e), (state, None))),
                    }
                }
            },
        ))
    }

    fn slice_chunk(&mut self, chunk: Bytes) -> Option<Bytes> {
        let chunk_len = chunk.len() as u64;
        let chunk_start = self.current_offset;
        let chunk_end = chunk_start + chunk_len;
        self.current_offset = chunk_end;

        let Some((req_start, req_end)) = self.req_range else {
            return Some(chunk);
        };

        if chunk_end <= req_start || chunk_start > req_end {
            return None;
        }

        let slice_start = req_start.saturating_sub(chunk_start) as usize;
        let slice_end = ((req_end + 1).min(chunk_end) - chunk_start) as usize;

        (slice_start < slice_end).then(|| chunk.slice(slice_start..slice_end))
    }
}

async fn fetch_segment_stream(
    client: &Client,
    seg: &HlsSegment,
) -> Result<SegmentByteStream, reqwest::Error> {
    let mut req = client.get(&seg.url);
    if let Some((start, end)) = seg.byte_range {
        req = req.header("Range", format!("bytes={start}-{end}"));
    }

    let res = req.send().await?.error_for_status()?;
    Ok(Box::pin(res.bytes_stream()))
}
