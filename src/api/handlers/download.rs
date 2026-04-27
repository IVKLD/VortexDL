use crate::api::{
    errors::ApiError,
    models::{ActionStatus, DownloadRequest},
    state::AppState,
    download_manager::ServerEvent,
};
use crate::{downloader, utils};
use axum::response::sse::{Event, Sse};
use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use futures_util::stream::Stream;
use std::convert::Infallible;

pub async fn queue_track(
    State(state): State<AppState>,
    Json(body): Json<DownloadRequest>,
) -> Result<impl IntoResponse, ApiError> {
    process_generic_download(state, body.url, Some("track")).await
}

pub async fn queue_playlist(
    State(state): State<AppState>,
    Json(body): Json<DownloadRequest>,
) -> Result<impl IntoResponse, ApiError> {
    process_generic_download(state, body.url, Some("playlist")).await
}

pub async fn queue_likes(
    State(state): State<AppState>,
    Json(body): Json<DownloadRequest>,
) -> Result<impl IntoResponse, ApiError> {
    process_generic_download(state, body.url, Some("likes")).await
}

pub async fn process_download(
    State(state): State<AppState>,
    Json(body): Json<DownloadRequest>,
) -> Result<impl IntoResponse, ApiError> {
    process_generic_download(state, body.url, None).await
}

async fn process_generic_download(
    state: AppState,
    url: String,
    expected_kind: Option<&'static str>,
) -> Result<impl IntoResponse, ApiError> {
    if url.is_empty() {
        return Err(ApiError::bad_request("Field `url` must not be empty"));
    }

    let kind = expected_kind.unwrap_or("universal");
    tracing::info!("Received {} download request for URL: {}", kind, url);

    state.download_manager.broadcast_event(ServerEvent::OperationStarted {
        url: url.clone(),
        kind: kind.to_string(),
    });

    let state_clone = state.clone();
    let url_clone = url.clone();

    tokio::spawn(async move {
        let mut final_kind = kind.to_string();
        let result = if let Some(k) = expected_kind {
            execute_download(&state_clone, &url_clone, k).await
        } else {
            match utils::soundcloud::resolve_url(&state_clone.client, &url_clone).await {
                Ok(res) => execute_download(&state_clone, &url_clone, &res.kind).await,
                Err(e) => Err(e),
            }
        };

        let status = match result {
            Ok(_) => "finished",
            Err(e) => {
                tracing::error!("Download failed for {}: {}", url_clone, e);
                "failed"
            }
        };

        state_clone.download_manager.broadcast_event(ServerEvent::OperationFinished {
            url: url_clone,
            kind: final_kind,
            status: status.to_string(),
        });
    });

    Ok((
        StatusCode::ACCEPTED,
        Json(ActionStatus {
            status: "queued",
            message: format!("Download started for: {}", url),
        }),
    ))
}

async fn execute_download(state: &AppState, url: &str, kind: &str) -> crate::models::Result<()> {
    match kind {
        "track" => {
            let res = utils::soundcloud::resolve_url(&state.client, url).await?;
            downloader::download_track(
                state.storage.clone(),
                &state.client,
                res.id,
                &state.output_dir,
                Some(state.download_manager.clone()),
            ).await
        }
        "playlist" => {
            downloader::download_playlist(
                state.storage.clone(),
                &state.client,
                url,
                &state.output_dir,
                Some(state.download_manager.clone()),
            ).await?;
            Ok(())
        }
        "user" | "likes" => {
            downloader::download_likes(
                state.storage.clone(),
                &state.client,
                url,
                &state.output_dir,
                state.config.clone(),
                Some(state.download_manager.clone()),
            ).await?;
            Ok(())
        }
        _ => Err(format!("Unsupported resource kind: {}", kind).into()),
    }
}

pub async fn get_download_queue(State(state): State<AppState>) -> impl IntoResponse {
    let queue = state.download_manager.get_queue().await;
    Json(queue)
}

pub async fn download_events(
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let mut rx = state.download_manager.subscribe();

    let stream = async_stream::stream! {
        let queue = state.download_manager.get_queue().await;
        for item in queue {
            if matches!(item.status, crate::api::download_manager::DownloadStatus::Queued | crate::api::download_manager::DownloadStatus::Downloading) {
                yield Ok(Event::default().json_data(ServerEvent::TrackUpdate { item }).unwrap());
            }
        }

        loop {
            match rx.recv().await {
                Ok(event) => {
                    yield Ok(Event::default().json_data(event).unwrap());
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {
                    continue;
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                    break;
                }
            }
        }
    };

    Sse::new(stream).keep_alive(axum::response::sse::KeepAlive::default())
}
