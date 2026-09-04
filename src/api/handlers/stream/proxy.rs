use axum::{
    http::{HeaderValue, StatusCode, header},
    response::Response,
};

use crate::api::{
    errors::ApiError, handlers::stream::resolve::resolve_stream_url, state::AppState,
};

pub async fn proxy_audio_stream(
    state: &AppState,
    id: i64,
    url_param: Option<String>,
    range_header: Option<HeaderValue>,
) -> Result<Response, ApiError> {
    let stream_url = resolve_stream_url(state, id, url_param.clone()).await?;
    let settings = state.settings.read().await.clone();
    let proxy_url = settings.network.get_proxy_url();
    let client = yt_audio_downloader::create_http_client_with_proxy(proxy_url);

    let mut upstream_res =
        send_upstream_request(&client, &stream_url, range_header.as_ref()).await?;

    if upstream_res.status().is_client_error() {
        state.cache.streams.write().await.remove(&id);
        if let Ok(fresh_url) = resolve_stream_url(state, id, url_param).await
            && let Ok(res) = send_upstream_request(&client, &fresh_url, range_header.as_ref()).await
            && (res.status().is_success() || res.status().as_u16() == 206)
        {
            upstream_res = res;
        }
    }

    build_streaming_response(upstream_res)
}

async fn send_upstream_request(
    client: &reqwest::Client,
    url: &str,
    range_header: Option<&HeaderValue>,
) -> Result<reqwest::Response, ApiError> {
    let mut req = client.get(url).header(
        reqwest::header::USER_AGENT,
        yt_audio_downloader::select_user_agent_for_url(url),
    );

    if let Some(range) = range_header
        && let Ok(range_str) = range.to_str()
    {
        req = req.header(reqwest::header::RANGE, range_str);
    }

    Ok(req.send().await?)
}

fn build_streaming_response(upstream_res: reqwest::Response) -> Result<Response, ApiError> {
    let status = StatusCode::from_u16(upstream_res.status().as_u16()).unwrap_or(StatusCode::OK);
    let mut response_builder = Response::builder().status(status);

    for (name, value) in upstream_res.headers() {
        if matches!(
            name,
            &reqwest::header::CONTENT_TYPE
                | &reqwest::header::CONTENT_LENGTH
                | &reqwest::header::CONTENT_RANGE
                | &reqwest::header::ACCEPT_RANGES
        ) {
            response_builder = response_builder.header(name.as_str(), value.as_bytes());
        }
    }

    if upstream_res
        .headers()
        .get(reqwest::header::ACCEPT_RANGES)
        .is_none()
    {
        response_builder = response_builder.header(header::ACCEPT_RANGES, "bytes");
    }

    let body = axum::body::Body::from_stream(upstream_res.bytes_stream());
    Ok(response_builder.body(body)?)
}
