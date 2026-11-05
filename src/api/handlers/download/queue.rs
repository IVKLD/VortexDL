use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};

use crate::{
    api::{
        errors::ApiError,
        types::{DownloadRequest, DownloadStartResponse},
    },
    app::AppState,
    downloader::DownloadItem,
};

#[utoipa::path(
    method(post),
    path = "/api/download",
    request_body = DownloadRequest,
    responses(
        (status = 202, description = "Download added to queue", body = DownloadStartResponse)
    )
)]
pub async fn start_download(
    State(state): State<AppState>,
    Json(body): Json<DownloadRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let status = state.download_service().enqueue(body.url).await?;
    Ok((StatusCode::ACCEPTED, Json(status)))
}

#[utoipa::path(
    method(get),
    path = "/api/download/syncing",
    responses(
        (status = 200, description = "Get list of currently syncing URLs", body = Vec<String>)
    )
)]
pub async fn get_syncing_urls(State(state): State<AppState>) -> impl IntoResponse {
    Json(state.download_queue.get_reserved_urls())
}

#[utoipa::path(
    method(get),
    path = "/api/download/queue",
    responses(
        (status = 200, description = "Get list of active downloads in queue", body = Vec<DownloadItem>)
    )
)]
pub async fn get_download_queue(State(state): State<AppState>) -> impl IntoResponse {
    Json(state.download_queue.get_queue())
}

#[utoipa::path(
    method(delete),
    path = "/api/download/queue/{id}",
    params(
        ("id" = i64, Path, description = "Item ID to remove from queue")
    ),
    responses(
        (status = 200, description = "Item removed successfully")
    )
)]
pub async fn remove_from_queue(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    state.download_queue.remove_task(id);
    StatusCode::OK
}
