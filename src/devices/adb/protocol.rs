use std::collections::HashSet;

use tokio::io::{AsyncReadExt, AsyncWriteExt};

use super::client::raw;
use crate::devices::error::DeviceError;

pub fn server_addr() -> String {
    let host = std::env::var("ADB_SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = std::env::var("ADB_SERVER_PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(5037);
    format!("{host}:{port}")
}

pub async fn ensure_adb_server_running() -> Result<(), DeviceError> {
    let output = raw(&["start-server"]).await?;

    if !output.status.success() {
        return Err(DeviceError::Other(anyhow::anyhow!(
            "adb start-server failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    Ok(())
}

pub async fn connect_to_adb() -> Result<tokio::net::TcpStream, DeviceError> {
    let addr = server_addr();
    match tokio::net::TcpStream::connect(&addr).await {
        Ok(s) => Ok(s),
        Err(_) => {
            ensure_adb_server_running().await?;
            tokio::net::TcpStream::connect(&addr).await.map_err(|e| {
                DeviceError::Other(anyhow::anyhow!(
                    "Failed to connect to ADB server at {addr} after start-server: {e}"
                ))
            })
        }
    }
}

pub async fn send_adb_cmd(
    stream: &mut tokio::net::TcpStream,
    cmd: &str,
) -> Result<(), DeviceError> {
    let payload = format!("{:04x}{}", cmd.len(), cmd);
    stream
        .write_all(payload.as_bytes())
        .await
        .map_err(|e| DeviceError::Other(anyhow::anyhow!("Failed to write ADB command: {e}")))?;

    let mut status = [0u8; 4];
    stream
        .read_exact(&mut status)
        .await
        .map_err(|e| DeviceError::Other(anyhow::anyhow!("Failed to read ADB status: {e}")))?;

    if &status == b"OKAY" {
        Ok(())
    } else if &status == b"FAIL" {
        let payload = read_adb_hex_packet(stream).await?;
        let err_msg = String::from_utf8_lossy(&payload).into_owned();
        Err(DeviceError::Other(anyhow::anyhow!(
            "ADB server returned FAIL: {err_msg}"
        )))
    } else {
        Err(DeviceError::Other(anyhow::anyhow!(
            "ADB server returned unexpected status: {:?}",
            status
        )))
    }
}

/// What: Reads a single packet from an ADB daemon TCP connection.
/// Flow: Called by ADB streaming functions (like device tracking) whenever the daemon sends new data.
/// Why:  The ADB host protocol prefixes every payload with a 4-character hex length string (e.g. "0012").
///       We read the exact length first, parse the hex, then read the remaining payload bytes.
pub async fn read_adb_hex_packet(
    stream: &mut tokio::net::TcpStream,
) -> Result<Vec<u8>, DeviceError> {
    let mut len_buf = [0u8; 4];
    stream
        .read_exact(&mut len_buf)
        .await
        .map_err(|e| DeviceError::Other(anyhow::anyhow!("Failed to read packet length: {e}")))?;
    let len_str = std::str::from_utf8(&len_buf)
        .map_err(|e| DeviceError::Other(anyhow::anyhow!("Length not valid UTF-8: {e}")))?;
    let len = usize::from_str_radix(len_str, 16).map_err(|e| {
        DeviceError::Other(anyhow::anyhow!("Length not valid hex '{len_str}': {e}"))
    })?;

    let mut payload = vec![0u8; len];
    stream
        .read_exact(&mut payload)
        .await
        .map_err(|e| DeviceError::Other(anyhow::anyhow!("Failed to read packet payload: {e}")))?;
    Ok(payload)
}

pub async fn subscribe_track_devices() -> Result<tokio::net::TcpStream, DeviceError> {
    let mut stream = connect_to_adb().await?;
    send_adb_cmd(&mut stream, "host:track-devices").await?;
    Ok(stream)
}

/// What: Awaits the next device state event packet and parses it into a set of device IDs.
/// Flow: Called in an infinite loop by the ADB tracker task (`start_device_tracker`).
/// Why:  The ADB daemon keeps the socket open and pushes updates whenever an Android device connects or disconnects.
pub async fn read_device_list_update(
    stream: &mut tokio::net::TcpStream,
) -> Result<HashSet<String>, DeviceError> {
    let payload = read_adb_hex_packet(stream).await?;
    let payload_str = String::from_utf8(payload)
        .map_err(|e| DeviceError::Other(anyhow::anyhow!("Payload not valid UTF-8: {e}")))?;

    Ok(super::client::parse_device_list(&payload_str))
}
