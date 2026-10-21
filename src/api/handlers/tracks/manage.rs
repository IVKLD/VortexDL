use std::time::SystemTime;

use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use tokio::task::spawn_blocking;

use crate::{
    api::errors::ApiError,
    app::AppState,
    domain::AudioFormat,
    infra::db::{DeletedTrackRecord, record_deleted_tracks},
};

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
    let mut storage = state.library.write().await;
    let removed = storage.remove_tracks(ids).await?;
    if !removed.is_empty() {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let records = removed
            .into_iter()
            .map(|t| {
                let format = AudioFormat::from_path(&t.path).to_string();
                DeletedTrackRecord {
                    id: t.metadata.id,
                    title: t.metadata.title,
                    artist: t.metadata.artist,
                    format,
                    size: t.size,
                    deleted_at: now,
                }
            })
            .collect::<Vec<_>>();
        let _ = spawn_blocking(move || {
            let _ = record_deleted_tracks(&records);
        })
        .await;
    }
    Ok(StatusCode::OK)
}
