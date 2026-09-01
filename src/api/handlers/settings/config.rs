use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};

use crate::{
    api::{errors::ApiError, state::AppState},
    settings::UserSettings,
};

#[utoipa::path(
    method(get),
    path = "/api/settings",
    responses(
        (status = 200, description = "Get current settings", body = UserSettings)
    )
)]
pub async fn get_settings(State(state): State<AppState>) -> impl IntoResponse {
    let current = state.settings.read().await;
    Json(&*current).into_response()
}

#[utoipa::path(
    method(post),
    path = "/api/settings",
    request_body = UserSettings,
    responses(
        (status = 200, description = "Settings updated successfully")
    )
)]
pub async fn update_settings(
    State(state): State<AppState>,
    Json(payload): Json<UserSettings>,
) -> Result<impl IntoResponse, ApiError> {
    state.settings.update(payload).await?;
    Ok(StatusCode::OK)
}
