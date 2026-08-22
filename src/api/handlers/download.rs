use axum::{
    Json,
    extract::{
        Path, State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    http::StatusCode,
    response::IntoResponse,
};

use crate::{
    api::{
        download_manager::{DownloadItem, MessageLevel, ServerEvent},
        errors::{ApiError, ErrorCode},
        state::AppState,
        types::{ApiStatus, DownloadRequest, DownloadStartResponse},
    },
    downloader,
};

async fn run_pipeline_bg(state: AppState, url: url::Url) {
    let ctx = downloader::Context::from_state(&state).with_dm(state.download_manager.clone());

    if let Err(e) = downloader::run_download_pipeline(&ctx, &url).await {
        state.download_manager.broadcast_event(ServerEvent::Error {
            message: format!("Download failed: {e}"),
        });
    }

    state.download_manager.release_url(&url);
    state
        .download_manager
        .broadcast_event(ServerEvent::SyncFinished {
            url: Some(url.to_string()),
        });
}

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

    if !state.download_manager.reserve_url(&url) {
        return Err(ApiError::conflict("This URL is already being processed")
            .with_code(ErrorCode::AlreadyProcessing));
    }

    state
        .download_manager
        .broadcast_event(ServerEvent::SyncStarted {
            url: url.to_string(),
        });

    let status = DownloadStartResponse {
        status: ApiStatus::Queued,
        message: format!("Started for: {url}"),
    };

    tokio::task::spawn_blocking(move || {
        tokio::runtime::Handle::current().block_on(run_pipeline_bg(state, url));
    });

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
    Json(state.download_manager.get_reserved_urls())
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
        (status = 101, description = "WebSocket upgrade for active downloads")
    )
)]
pub async fn download_events(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_download_events(socket, state))
}

async fn send_event(socket: &mut WebSocket, event: &ServerEvent) -> bool {
    if let Ok(msg_str) = serde_json::to_string(event) {
        socket.send(Message::Text(msg_str.into())).await.is_ok()
    } else {
        true
    }
}

async fn handle_download_events(mut socket: WebSocket, state: AppState) {
    let queue = state.download_manager.get_queue();
    let mut rx = state.download_manager.subscribe();

    let welcome = ServerEvent::Message {
        message: "Connected to event stream".to_string(),
        level: MessageLevel::Info,
    };
    if !send_event(&mut socket, &welcome).await {
        return;
    }

    for item in queue {
        let update = ServerEvent::TrackUpdate {
            item: Box::new(item),
        };
        if !send_event(&mut socket, &update).await {
            return;
        }
    }

    loop {
        match rx.recv().await {
            Ok(event) => {
                if !send_event(&mut socket, &event).await {
                    break;
                }
            }
            Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {
                let queue = state.download_manager.get_queue();
                for item in queue {
                    let update = ServerEvent::TrackUpdate {
                        item: Box::new(item),
                    };
                    if !send_event(&mut socket, &update).await {
                        return;
                    }
                }
                continue;
            }
            Err(_) => break,
        }
    }
}
