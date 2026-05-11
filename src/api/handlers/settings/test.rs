use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
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

    // Using the generic get method from soundcloud-rs Client
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
            Json(format!("SoundCloud verification failed: {}", e)),
        )
            .into_response(),
    }
}
