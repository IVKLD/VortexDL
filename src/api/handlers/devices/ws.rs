use std::time::Duration;

use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
};
use serde_json::to_string;
use tokio::{select, time::interval};

use crate::devices::list_devices;

async fn handle_socket(mut socket: WebSocket) {
    let mut last_devices = Vec::new();
    let mut tick_interval = interval(Duration::from_secs(2));

    loop {
        select! {
            _ = tick_interval.tick() => {
                if let Ok(devices) = list_devices().await {
                    let mut list: Vec<String> = devices.into_iter().collect();
                    list.sort();

                    if list != last_devices {
                        last_devices = list.clone();
                        let msg_str = match to_string(&list) {
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
pub async fn devices_ws(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(handle_socket)
}
