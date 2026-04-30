use crate::api::{
    errors::ApiError,
    models::{ActionStatus, DownloadRequest},
    state::AppState,
    download_manager::ServerEvent,
};
use crate::downloader;
use axum::response::sse::{Event, Sse};
use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use futures_util::stream::Stream;
use std::convert::Infallible;

pub async fn start_download(
    State(state): State<AppState>,
    Json(body): Json<DownloadRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let url = body.url;
    if url.is_empty() {
        return Err(ApiError::bad_request("Empty URL"));
    }

    tracing::info!("Download request: {url}");

    state.download_manager.broadcast_event(ServerEvent::OperationStarted {
        url: url.clone(),
        kind: "universal".to_string(),
    });

    let s = state.clone();
    let u = url.clone();

    tokio::spawn(async move {
        let res = downloader::dispatch_download(
            &u,
            s.storage.clone(),
            &s.client,
            &s.output_dir,
            s.config.clone(),
            Some(s.download_manager.clone()),
        ).await;

        let status = if res.is_ok() { "finished" } else { "failed" };
        
        if let Err(ref e) = res {
            tracing::error!("Download failed for {u}: {e}");
        }

        s.download_manager.broadcast_event(ServerEvent::OperationFinished {
            url: u,
            kind: "universal".to_string(),
            status: status.to_string(),
        });
    });

    Ok((
        StatusCode::ACCEPTED,
        Json(ActionStatus {
            status: "queued",
            message: format!("Started for: {url}"),
        }),
    ))
}


pub async fn get_download_queue(State(state): State<AppState>) -> impl IntoResponse {
    Json(state.download_manager.get_queue().await)
}

pub async fn download_events(State(state): State<AppState>) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let mut rx = state.download_manager.subscribe();
    let s = state.clone();

    let stream = async_stream::stream! {
        let queue = s.download_manager.get_queue().await;
        for item in queue {
            if matches!(item.status, crate::api::download_manager::DownloadStatus::Queued | crate::api::download_manager::DownloadStatus::Downloading) {
                yield Ok(Event::default().json_data(ServerEvent::TrackUpdate { item }).unwrap());
            }
        }
        loop {
            if let Ok(event) = rx.recv().await { yield Ok(Event::default().json_data(event).unwrap()); }
            else { break; }
        }
    };

    Sse::new(stream).keep_alive(axum::response::sse::KeepAlive::default())
}
