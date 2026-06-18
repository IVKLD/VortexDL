use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use crate::{api::{errors::{ApiError, ErrorCode}, state::AppState}, settings::UserSettings};

pub mod test;

pub async fn get_settings(State(state): State<AppState>) -> impl IntoResponse {
    let current = state.settings.read().await;
    Json(&*current).into_response()
}

pub async fn update_settings(
    State(state): State<AppState>,
    Json(payload): Json<UserSettings>,
) -> Result<impl IntoResponse, ApiError> {
    state.settings.update(payload).await
        .map(|_| StatusCode::OK)
        .map_err(|e| ApiError::internal(format!("Failed to update settings: {e}"))
            .with_code(ErrorCode::DatabaseError))
}
