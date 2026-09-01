pub mod proxy;
pub mod resolve;
pub mod types;

use axum::{
    extract::{Path, Query, Request, State},
    http::header,
    response::{IntoResponse, Response},
};
pub use proxy::proxy_audio_stream;
use tower::ServiceExt;
use tower_http::services::ServeFile;
pub use types::StreamQuery;

use crate::api::{errors::ApiError, state::AppState};

#[utoipa::path(
    method(get),
    path = "/api/stream/{id}",
    params(
        ("id" = i64, Path, description = "Track ID to stream"),
        ("url" = Option<String>, Query, description = "Track permalink or source URL")
    ),
    responses(
        (status = 200, description = "Audio stream of track file or remote audio stream"),
        (status = 206, description = "Partial content audio stream")
    )
)]
pub async fn stream_audio(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Query(params): Query<StreamQuery>,
    req: Request,
) -> Result<Response, ApiError> {
    if let Some(path) = state
        .storage
        .read()
        .await
        .tracks
        .get(&id)
        .map(|t| t.path.clone())
        && path.exists()
    {
        let res = ServeFile::new(path).oneshot(req).await?;
        return Ok(res.into_response());
    }

    let range_header = req.headers().get(header::RANGE).cloned();
    proxy_audio_stream(&state, id, params.url, range_header).await
}
