use axum::{
    Json,
    extract::{
        State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    http::StatusCode,
    response::IntoResponse,
};
use serde::Deserialize;
use url::Url;
use utoipa::ToSchema;

pub use crate::infra::net::ProxyHealthResult as ProxyTestResult;
use crate::{api::errors::ApiError, app::AppState, infra::net};

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
    state.soundcloud.resolve_url(&payload.url).await?;
    Ok((
        StatusCode::OK,
        Json("SoundCloud URL is valid and accessible"),
    ))
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

        let mut rx = net::test_proxies(proxies, cached_client_id);
        while let Some(result) = rx.recv().await {
            if let Ok(msg_str) = serde_json::to_string(&result)
                && socket.send(Message::Text(msg_str.into())).await.is_err()
            {
                break;
            }
        }
    }
}
