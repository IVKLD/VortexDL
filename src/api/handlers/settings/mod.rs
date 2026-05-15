use axum::{Json, http::StatusCode, response::IntoResponse};

use crate::database::settings::{self, UserSettings};

pub(crate) mod test;

pub async fn get_settings() -> impl IntoResponse {
    match settings::get_settings() {
        Ok(s) => (StatusCode::OK, Json(s)).into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "DB Error").into_response(),
    }
}

pub async fn update_settings(Json(payload): Json<UserSettings>) -> impl IntoResponse {
    match settings::update_settings(&payload) {
        Ok(_) => StatusCode::OK.into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to update settings",
        )
            .into_response(),
    }
}
