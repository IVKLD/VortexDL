use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};

use crate::{api::state::AppState, settings::UserSettings};

pub mod test;

pub async fn get_settings(State(state): State<AppState>) -> impl IntoResponse {
    let current = state.settings.read().await.clone();
    (StatusCode::OK, Json(current)).into_response()
}

pub async fn update_settings(
    State(state): State<AppState>,
    Json(payload): Json<UserSettings>,
) -> impl IntoResponse {
    match state.settings.update(payload).await {
        Ok(_) => StatusCode::OK.into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to update settings",
        )
            .into_response(),
    }
}
