use std::time::Duration;

use axum::{
    extract::{
        State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    response::IntoResponse,
};

use crate::{app::AppState, devices::DeviceService};

async fn handle_socket(mut socket: WebSocket, device_service: DeviceService) {
    let mut last_devices = Vec::new();
    let mut tick_interval = tokio::time::interval(Duration::from_secs(2));

    loop {
        tokio::select! {
            _ = tick_interval.tick() => {
                if let Ok(devices) = device_service.list_devices().await {
                    let mut list: Vec<String> = devices.into_iter().collect();
                    list.sort();

                    if list != last_devices {
                        last_devices = list.clone();
                        let msg_str = match serde_json::to_string(&list) {
                            Ok(s) => s,
                            Err(_) => continue,
                        };
                        if socket.send(Message::Text(msg_str.into())).await.is_err() {
                            break;
                        }
                    }
                }
            }
            msg = socket.recv() => {
                match msg {
                    Some(Ok(_)) => {}
                    _ => break,
                }
            }
        }
    }
}

#[utoipa::path(
    method(get),
    path = "/api/devices/ws",
    responses(
        (status = 101, description = "WebSocket upgrade to stream connected ADB devices")
    )
)]
pub async fn devices_ws(State(state): State<AppState>, ws: WebSocketUpgrade) -> impl IntoResponse {
    let device_service = state.device_service();
    ws.on_upgrade(move |socket| handle_socket(socket, device_service))
}
