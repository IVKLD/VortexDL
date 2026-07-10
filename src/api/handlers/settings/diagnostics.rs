use axum::{
    Json,
    extract::{
        State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use soundcloud_rs::ClientBuilder;
use url::Url;
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
    #[schema(value_type = String)]
    pub url: Url,
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
#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ProxyTestResult {
    pub url: String,
    pub valid: bool,
    pub error: Option<String>,
}

#[utoipa::path(
    method(get),
    path = "/api/settings/test/proxy/ws",
    responses(
        (status = 101, description = "WebSocket upgrade to test multiple proxies concurrently with real-time feedback")
    )
)]
pub async fn test_proxy_ws(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_proxy_ws(socket, state))
}

async fn handle_proxy_ws(mut socket: WebSocket, state: AppState) {
    let proxies = match socket.recv().await {
        Some(Ok(Message::Text(text))) => serde_json::from_str::<Vec<String>>(&text).ok(),
        _ => None,
    };
    if let Some(proxies) = proxies {
        let cached_client_id = state
            .settings
            .read()
            .await
            .soundcloud
            .cached_client_id
            .clone();

        let mut tasks = tokio::task::JoinSet::new();

        for url in proxies {
            let cached_id = cached_client_id.clone();
            tasks.spawn(async move {
                let mut builder = ClientBuilder::new().with_proxy(url.clone());
                if let Some(id) = cached_id {
                    builder = builder.with_client_id(id);
                }
                match builder.build().await {
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

        while let Some(res) = tasks.join_next().await {
            if let Ok(Ok(msg_str)) = res.map(|result| serde_json::to_string(&result)) {
                let send_res = socket.send(Message::Text(msg_str.into())).await;
                if send_res.is_err() {
                    break;
                }
            }
        }
    }
}
