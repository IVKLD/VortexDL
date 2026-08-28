use axum::{
    extract::{
        State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    response::IntoResponse,
};

use crate::api::{
    download_manager::{MessageLevel, ServerEvent},
    state::AppState,
};

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
