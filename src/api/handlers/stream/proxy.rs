use axum::{
    body::Body,
    http::{HeaderValue, StatusCode},
    response::Response,
};
use reqwest::{
    Client, Response as ReqwestResponse,
    header::{ACCEPT_RANGES, CONTENT_LENGTH, CONTENT_RANGE, CONTENT_TYPE, RANGE, USER_AGENT},
};
use yt_audio_downloader::select_user_agent_for_url;

use super::resolve::resolve_stream_url;
use crate::{api::errors::ApiError, app::AppState};

pub async fn proxy_audio_stream(
    state: &AppState,
    id: i64,
    url_param: Option<String>,
    range_header: Option<HeaderValue>,
) -> Result<Response, ApiError> {
    let cached = resolve_stream_url(state, id, url_param.clone()).await?;
    let client = state.cache.get_or_create_client(cached.proxy.clone());

    let mut upstream_res =
        send_upstream_request(&client, &cached.url, range_header.as_ref()).await?;

    let status = upstream_res.status().as_u16();
    if matches!(status, 401 | 403 | 404 | 410) {
        state.cache.remove_stream(id);
        if let Ok(fresh) = resolve_stream_url(state, id, url_param).await {
            let fresh_client = state.cache.get_or_create_client(fresh.proxy);
            if let Ok(res) =
                send_upstream_request(&fresh_client, &fresh.url, range_header.as_ref()).await
                && (res.status().is_success() || res.status().as_u16() == 206)
            {
                upstream_res = res;
            }
        }
    }

    build_streaming_response(upstream_res)
}

async fn send_upstream_request(
    client: &Client,
    url: &str,
    range_header: Option<&HeaderValue>,
) -> Result<ReqwestResponse, ApiError> {
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

fn build_streaming_response(upstream_res: ReqwestResponse) -> Result<Response, ApiError> {
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
    Ok(response_builder.body(body)?)
}
