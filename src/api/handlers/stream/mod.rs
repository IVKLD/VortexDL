use axum::{
    extract::{Path, Query, Request, State},
    response::Response,
};
use serde::Deserialize;

use crate::{app::AppState, stream::StreamError};

#[derive(Debug, Deserialize, Clone)]
pub struct StreamQuery {
    pub url: Option<String>,
}

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
) -> Result<Response, StreamError> {
    state.stream_service().serve(id, params.url, req).await
}
