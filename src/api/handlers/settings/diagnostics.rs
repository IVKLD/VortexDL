use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use serde::{Deserialize, Serialize};
use soundcloud_rs::ClientBuilder;
use utoipa::ToSchema;

use crate::{
    api::{
        errors::{ApiError, ErrorCode},
        state::AppState,
    },
    utils::soundcloud::resolve_url,
};

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TestSoundCloudRequest {
    pub url: String,
}

#[utoipa::path(
    method(post),
    path = "/api/settings/test/soundcloud",
    request_body = TestSoundCloudRequest,
    responses(
        (status = 200, description = "SoundCloud URL is valid and accessible", body = String)
    )
)]
pub async fn test_soundcloud(
    State(state): State<AppState>,
    Json(payload): Json<TestSoundCloudRequest>,
) -> Result<impl IntoResponse, ApiError> {
    resolve_url(&state.client, &payload.url)
        .await
        .map(|_| {
            (
                StatusCode::OK,
                Json("SoundCloud URL is valid and accessible"),
            )
        })
        .map_err(|e| {
            ApiError::bad_request(format!("SoundCloud verification failed: {e}"))
                .with_code(ErrorCode::SoundCloudError)
        })
}

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TestProxiesRequest {
    pub proxy_urls: Vec<String>,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ProxyTestResult {
    pub url: String,
    pub valid: bool,
    pub error: Option<String>,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TestProxiesResponse {
    pub results: Vec<ProxyTestResult>,
}

#[utoipa::path(
    method(post),
    path = "/api/settings/test/proxy",
    request_body = TestProxiesRequest,
    responses(
        (status = 200, description = "Proxy test results", body = TestProxiesResponse)
    )
)]
pub async fn test_proxy(
    State(_state): State<AppState>,
    Json(payload): Json<TestProxiesRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let mut tasks = tokio::task::JoinSet::new();

    for url in payload.proxy_urls {
        tasks.spawn(async move {
            match ClientBuilder::new().with_proxy(url.clone()).build().await {
                Ok(client) if client.health_check().await => ProxyTestResult {
                    url,
                    valid: true,
                    error: None,
                },
                Ok(_) => ProxyTestResult {
                    url,
                    valid: false,
                    error: Some("Proxy is not able to reach SoundCloud API".to_string()),
                },
                Err(e) => ProxyTestResult {
                    url,
                    valid: false,
                    error: Some(format!("Failed to build client: {e}")),
                },
            }
        });
    }

    let mut results = Vec::new();
    while let Some(res) = tasks.join_next().await {
        if let Ok(result) = res {
            results.push(result);
        }
    }

    Ok((StatusCode::OK, Json(TestProxiesResponse { results })))
}
