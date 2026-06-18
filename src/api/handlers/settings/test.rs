use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use serde::Deserialize;
use soundcloud_rs::ClientBuilder;

use crate::{
    api::{
        errors::{ApiError, ErrorCode},
        state::AppState,
    },
    utils::soundcloud::resolve_url,
};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TestSoundCloudRequest {
    pub url: String,
}

pub async fn test_soundcloud_url(
    State(state): State<AppState>,
    Json(payload): Json<TestSoundCloudRequest>,
) -> Result<impl IntoResponse, ApiError> {
    resolve_url(&state.client, &payload.url)
        .await
        .map(|_| (StatusCode::OK, Json("SoundCloud URL is valid and accessible")))
        .map_err(|e| ApiError::bad_request(format!("SoundCloud verification failed: {e}"))
            .with_code(ErrorCode::SoundCloudError))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TestProxyRequest {
    pub proxy_url: String,
}

pub async fn test_proxy(
    State(_state): State<AppState>,
    Json(payload): Json<TestProxyRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let client = ClientBuilder::new()
        .with_proxy(payload.proxy_url)
        .build()
        .await
        .map_err(|e| {
            ApiError::bad_request(format!("Proxy connection failed: {e}"))
                .with_code(ErrorCode::NetworkError)
        })?;

    if client.health_check().await {
        Ok((
            StatusCode::OK,
            Json("Proxy is valid and SoundCloud API is reachable"),
        ))
    } else {
        Err(ApiError::bad_request("Proxy is not able to reach SoundCloud API")
            .with_code(ErrorCode::NetworkError))
    }
}
