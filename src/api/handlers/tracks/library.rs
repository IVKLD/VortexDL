use axum::{extract::State, http::StatusCode, response::IntoResponse};

use crate::{
    api::{errors::ApiError, state::AppState},
    storage::MusicStorage,
};

#[utoipa::path(
    method(post),
    path = "/api/library/reindex",
    responses(
        (status = 200, description = "Reindexing triggered successfully")
    )
)]
pub async fn reindex_library(State(state): State<AppState>) -> Result<impl IntoResponse, ApiError> {
    let base_path = state.settings.output_path().await;
    let tracks = MusicStorage::scan_library(&base_path).await;
    state.storage.write().await.update_tracks(tracks);
    Ok(StatusCode::OK)
}
