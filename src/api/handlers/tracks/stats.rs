use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};

use crate::{api::errors::ApiError, app::AppState, library::LibraryStats};

pub type LibraryStatsResponse = LibraryStats;

#[utoipa::path(
    method(get),
    path = "/api/downloads/stats",
    responses(
        (status = 200, description = "Get library and download statistics", body = LibraryStatsResponse)
    )
)]
pub async fn get_library_stats(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, ApiError> {
    let stats = state.library_service().get_stats().await;
    Ok(Json(stats))
}

#[utoipa::path(
    method(delete),
    path = "/api/downloads/stats",
    responses(
        (status = 200, description = "Cleared deleted tracks history")
    )
)]
pub async fn clear_deleted_stats(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, ApiError> {
    state.library_service().clear_deleted_stats().await?;
    Ok(StatusCode::OK)
}
