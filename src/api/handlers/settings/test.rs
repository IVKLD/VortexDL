use std::time::Duration;

use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use reqwest::{Client, Proxy};
use serde::{Deserialize, Serialize};

use crate::api::state::AppState;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TestSoundCloudRequest {
    pub url: String,
}

#[derive(Serialize)]
struct ResolveQuery<'a> {
    url: &'a str,
}

pub async fn test_soundcloud_url(
    State(state): State<AppState>,
    Json(payload): Json<TestSoundCloudRequest>,
) -> impl IntoResponse {
    let query = ResolveQuery { url: &payload.url };

    match state
        .client
        .get::<_, serde_json::Value>("resolve", Some(&query))
        .await
    {
        Ok(_) => (
            StatusCode::OK,
            Json("SoundCloud URL is valid and accessible"),
        )
            .into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(format!("SoundCloud verification failed: {e}")),
        )
            .into_response(),
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TestProxyRequest {
    pub proxy_url: String,
}

pub async fn test_proxy(
    State(state): State<AppState>,
    Json(payload): Json<TestProxyRequest>,
) -> impl IntoResponse {
    let proxy = match Proxy::all(&payload.proxy_url) {
        Ok(p) => p,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(format!("Invalid proxy URL: {e}")),
            )
                .into_response();
        }
    };

    let client = match Client::builder()
        .proxy(proxy)
        .timeout(Duration::from_secs(5))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(format!("Failed to build HTTP client: {e}")),
            )
                .into_response();
        }
    };

    let client_id = {
        let settings = state.settings.read().await;
        settings.soundcloud.cached_client_id.clone()
    };

    let test_url = match &client_id {
        Some(id) => format!(
            "https://api-v2.soundcloud.com/resolve?url=https://soundcloud.com/soundcloud&client_id={id}"
        ),
        None => "https://api-v2.soundcloud.com/resolve?url=https://soundcloud.com/soundcloud"
            .to_string(),
    };

    match client.get(&test_url).send().await {
        Ok(resp) if resp.status().is_success() => (
            StatusCode::OK,
            Json("Proxy is valid and SoundCloud API is reachable"),
        )
            .into_response(),
        Ok(resp) if resp.status() == StatusCode::UNAUTHORIZED => (
            StatusCode::OK,
            Json("Proxy is valid and SoundCloud API is reachable"),
        )
            .into_response(),
        Ok(resp) => (
            StatusCode::BAD_REQUEST,
            Json(format!("Proxy returned status: {}", resp.status())),
        )
            .into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(format!("Proxy connection failed: {e}")),
        )
            .into_response(),
    }
}
