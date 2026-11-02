use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};

use crate::{api::errors::ApiError, app::AppState};

#[utoipa::path(
    method(delete),
    path = "/api/downloads",
    request_body = Vec<i64>,
    responses(
        (status = 200, description = "Tracks deleted successfully")
    )
)]
pub async fn remove_tracks(
    State(state): State<AppState>,
    Json(ids): Json<Vec<i64>>,
) -> Result<impl IntoResponse, ApiError> {
    state.library_service().remove_tracks(ids).await?;
    Ok(StatusCode::OK)
}
