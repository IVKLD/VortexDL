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
        errors::{ApiError, ErrorCode},
        state::AppState,
        types::{ApiStatus, DownloadRequest, DownloadStartResponse},
    },
    downloader,
};

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

        if let Err(e) = downloader::download(&ctx, &url).await {
            tracing::error!("Download failed for {url}: {e}");
        }

        state.download_manager.release_url(&url);
    });

    Ok((StatusCode::ACCEPTED, Json(status)))
}

pub async fn get_download_queue(State(state): State<AppState>) -> impl IntoResponse {
    Json(state.download_manager.get_queue())
}

pub async fn remove_from_queue(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    state.download_manager.remove_task(id);
    StatusCode::OK
}

pub async fn download_events(
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let mut rx = state.download_manager.subscribe();
    let dm = state.download_manager.clone();

    let stream = async_stream::stream! {
        if let Ok(evt) = Event::default().json_data(ServerEvent::Message {
            message: "Connected to event stream".to_string(),
            level: "info".to_string()
        }) {
            yield Ok(evt);
        }

        let queue = dm.get_queue();
        for item in queue {
            if matches!(item.status, DownloadStatus::Queued | DownloadStatus::Downloading) {
                let res = Event::default().json_data(ServerEvent::TrackUpdate { item });
                if let Ok(evt) = res {
                    yield Ok(evt);
                }
            }
        }
        loop {
            match rx.recv().await {
                Ok(event) => {
                    if let Ok(evt) = Event::default().json_data(event) {
                        yield Ok(evt);
                    }
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                Err(_) => break,
            }
        }
    };

    Sse::new(stream).keep_alive(KeepAlive::default())
}
