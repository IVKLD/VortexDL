use axum::{extract::State, http::StatusCode, response::IntoResponse};

use crate::{api::errors::ApiError, app::AppState, library};

#[utoipa::path(
    method(post),
    path = "/api/library/reindex",
    responses(
        (status = 200, description = "Reindexing triggered successfully")
    )
)]
pub async fn reindex_library(State(state): State<AppState>) -> Result<impl IntoResponse, ApiError> {
    let base_path = state.settings.read().await.downloads.output_path.clone();
    let tracks = library::scan_directory(&base_path).await;
    state.library.write().await.update_tracks(tracks);
    Ok(StatusCode::OK)
}
