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
        download_manager::{DownloadStatus, ServerEvent},
        errors::ApiError,
        models::{ActionStatus, DownloadRequest},
        state::AppState,
    },
    downloader,
    models::SyncMode,
};

pub async fn start_download(
    State(state): State<AppState>,
    Json(body): Json<DownloadRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let url = body.url;
    if url.is_empty() {
        return Err(ApiError::bad_request("Empty URL"));
    }

    tracing::info!("Download request: {url}");

    if !state.download_manager.reserve_url(&url).await {
        return Err(ApiError::conflict("This URL is already being processed"));
    }

    let status = ActionStatus {
        status: "queued",
        message: format!("Started for: {url}"),
    };

    spawn(async move {
        let ctx = downloader::Context {
            storage: state.storage.clone(),
            client: state.client.clone(),
            config: state.config.clone(),
            dm: Some(state.download_manager.clone()),
        };

        if let Err(e) = downloader::dispatch_download(&url, SyncMode::Silent, &ctx).await {
            tracing::error!("Download failed for {url}: {e}");
        }

        state.download_manager.release_url(&url).await;
    });

    Ok((StatusCode::ACCEPTED, Json(status)))
}

pub async fn get_download_queue(State(state): State<AppState>) -> impl IntoResponse {
    Json(state.download_manager.get_queue().await)
}

pub async fn remove_from_queue(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    state.download_manager.remove_task(id).await;
    StatusCode::OK
}

pub async fn download_events(
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let mut rx = state.download_manager.subscribe();
    let state = state.clone();

    let stream = async_stream::stream! {
        let queue = state.download_manager.get_queue().await;
        for item in queue {
            if matches!(item.status, DownloadStatus::Queued | DownloadStatus::Downloading) {
                yield Ok(Event::default().json_data(ServerEvent::TrackUpdate { item }).unwrap());
            }
        }
        loop {
            if let Ok(event) = rx.recv().await { yield Ok(Event::default().json_data(event).unwrap()); }
            else { break; }
        }
    };

    Sse::new(stream).keep_alive(KeepAlive::default())
}
