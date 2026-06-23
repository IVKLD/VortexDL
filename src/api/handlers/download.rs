use std::convert::Infallible;

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::{
        IntoResponse,
        sse::{Event, KeepAlive, Sse},
    },
};
use futures_util::stream::Stream;
use tokio::spawn;

use crate::{
    api::{
        download_manager::{DownloadItem, DownloadStatus, ServerEvent},
        errors::{ApiError, ErrorCode},
        state::AppState,
        types::{ApiStatus, DownloadRequest, DownloadStartResponse},
    },
    downloader,
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
    let url = body.url;
    if url.is_empty() {
        return Err(ApiError::bad_request("Empty URL").with_code(ErrorCode::EmptyUrl));
    }

    tracing::info!("Download request: {url}");

    if !state.download_manager.reserve_url(&url) {
        return Err(ApiError::conflict("This URL is already being processed")
            .with_code(ErrorCode::AlreadyProcessing));
    }

    let status = DownloadStartResponse {
        status: ApiStatus::Queued,
        message: format!("Started for: {url}"),
    };

    spawn(async move {
        let ctx = downloader::Context::from_state(&state).with_dm(state.download_manager.clone());

        if let Err(e) = downloader::run_download_pipeline(&ctx, &url).await {
            tracing::error!("Download failed for {url}: {e}");
        }

        state.download_manager.release_url(&url);
    });

    Ok((StatusCode::ACCEPTED, Json(status)))
}

#[utoipa::path(
    method(get),
    path = "/api/download/queue",
    responses(
        (status = 200, description = "Get list of active downloads in queue", body = Vec<DownloadItem>)
    )
)]
pub async fn get_download_queue(State(state): State<AppState>) -> impl IntoResponse {
    Json(state.download_manager.get_queue())
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
    state.download_manager.remove_task(id);
    StatusCode::OK
}

#[utoipa::path(
    method(get),
    path = "/api/download/events",
    responses(
        (status = 200, description = "SSE stream for active downloads", body = String)
    )
)]
pub async fn download_events(
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let queue = state.download_manager.get_queue();
    let mut rx = state.download_manager.subscribe();

    let stream = async_stream::stream! {
        if let Ok(evt) = Event::default().json_data(ServerEvent::Message {
            message: "Connected to event stream".to_string(),
            level: "info".to_string()
        }) {
            yield Ok(evt);
        }

        for item in queue {
            if matches!(item.status, DownloadStatus::Queued | DownloadStatus::Downloading) {
                let res = Event::default().json_data(ServerEvent::TrackUpdate { item });
                if let Ok(evt) = res {
                    yield Ok(evt);
                }
            }
        }
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(15));
        interval.tick().await;

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    if let Ok(evt) = Event::default().json_data(ServerEvent::Message {
                        message: "ping".to_string(),
                        level: "ping".to_string()
                    }) {
                        yield Ok(evt);
                    }
                }
                res = rx.recv() => {
                    match res {
                        Ok(event) => {
                            if let Ok(evt) = Event::default().json_data(event) {
                                yield Ok(evt);
                            }
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                        Err(_) => break,
                    }
                }
            }
        }
    };

    Sse::new(stream).keep_alive(KeepAlive::default())
}
