use axum::{extract::State, http::StatusCode, response::IntoResponse};

use crate::{api::errors::ApiError, app::AppState};

#[utoipa::path(
    method(post),
    path = "/api/library/reindex",
    responses(
        (status = 200, description = "Reindexing triggered successfully")
    )
)]
pub async fn reindex_library(State(state): State<AppState>) -> Result<impl IntoResponse, ApiError> {
    state.library_service().reindex().await;
    Ok(StatusCode::OK)
}
