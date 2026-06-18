use std::{fs::File, io::ErrorKind, path::Path};

use anyhow::{Result, anyhow};
use symphonia::core::{
    errors::Error, formats::FormatOptions, io::MediaSourceStream, meta::MetadataOptions,
    probe::Hint,
};
use tokio::fs;

pub async fn verify_file(path: impl AsRef<Path>, expected_size: u64) -> Result<()> {
    let path = path.as_ref();
    let final_size = fs::metadata(path).await?.len();

    if final_size < 10_000 || (expected_size > 0 && final_size != expected_size) {
        fs::remove_file(path).await.ok();
        return Err(anyhow!(
            "Integrity check failed: expected {expected_size} bytes, got on disk {final_size}"
        ));
    }

    let path_buf = path.to_path_buf();
    let is_valid = tokio::task::spawn_blocking(move || verify_audio_format(&path_buf))
        .await
        .unwrap_or(false);

    if !is_valid {
        fs::remove_file(path).await.ok();
        return Err(anyhow!(
            "Audio integrity check failed: file is corrupted or truncated"
        ));
    }

    Ok(())
}

fn verify_audio_format(path: &Path) -> bool {
    let Ok(file) = File::open(path) else {
        return false;
    };

    let mut hint = Hint::new();
    let mss = MediaSourceStream::new(Box::new(file), Default::default());

    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        hint.with_extension(ext);
    }

    let format_opts = FormatOptions {
        enable_gapless: false,
        ..Default::default()
    };

    let meta_opts = MetadataOptions::default();

    let Ok(mut probed) =
        symphonia::default::get_probe().format(&hint, mss, &format_opts, &meta_opts)
    else {
        return false;
    };

    let Some(track) = probed.format.default_track() else {
        return false;
    };

    let track_id = track.id;
    let mut packet_count = 0;

    loop {
        match probed.format.next_packet() {
            Ok(packet) => {
                if packet.track_id() == track_id {
                    packet_count += 1;
                }
            }
            Err(Error::IoError(err)) if err.kind() == ErrorKind::UnexpectedEof => {
                return packet_count > 100;
            }
            Err(_) => break,
        }
    }

    packet_count > 100
}
