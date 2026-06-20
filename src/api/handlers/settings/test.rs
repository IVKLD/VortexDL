use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
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

use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TestProxiesRequest {
    pub proxy_urls: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProxyTestResult {
    pub url: String,
    pub valid: bool,
    pub error: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TestProxiesResponse {
    pub results: Vec<ProxyTestResult>,
}

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
