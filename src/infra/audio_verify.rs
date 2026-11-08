use std::{fs::File, io::ErrorKind, path::Path};

use symphonia::core::{errors::Error, io::MediaSourceStream, probe::Hint};
use crate::infra::{FileService, error::AudioVerifyError};

pub async fn verify_audio_file(
    path: impl AsRef<Path>,
    expected_size: u64,
) -> Result<(), AudioVerifyError> {
    let path = path.as_ref();
    let final_size = FileService::file_size(path).await?;

    if final_size < 10_000 {
        let _ = FileService::remove_file_if_exists(path).await;
        return Err(AudioVerifyError::TooSmall {
            actual: final_size,
            min: 10_000,
        });
    }

    if expected_size > 0 && final_size != expected_size {
        let _ = FileService::remove_file_if_exists(path).await;
        return Err(AudioVerifyError::SizeMismatch {
            expected: expected_size,
            actual: final_size,
        });
    }

    let path_buf = path.to_path_buf();
    if !tokio::task::spawn_blocking(move || verify_format(&path_buf)).await.unwrap_or(false) {
        let _ = FileService::remove_file_if_exists(path).await;
        return Err(AudioVerifyError::CorruptedOrTruncated);
    }

    Ok(())
}

fn verify_format(path: &Path) -> bool {
    let Ok(file) = File::open(path) else {
        return false;
    };
    let mut hint = Hint::new();
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        hint.with_extension(ext);
    }
    let mss = MediaSourceStream::new(Box::new(file), Default::default());
    let Ok(mut probed) = symphonia::default::get_probe().format(
        &hint,
        mss,
        &Default::default(),
        &Default::default(),
    ) else {
        return false;
    };
    let Some(track) = probed.format.default_track() else {
        return false;
    };
    let Ok(mut decoder) =
        symphonia::default::get_codecs().make(&track.codec_params, &Default::default())
    else {
        return false;
    };

    let track_id = track.id;
    let mut packet_count = 0;

    loop {
        match probed.format.next_packet() {
            Ok(packet) => {
                if packet.track_id() == track_id {
                    if decoder.decode(&packet).is_err() {
                        return false;
                    }
                    packet_count += 1;
                    if packet_count > 100 {
                        return true;
                    }
                }
            }
            Err(Error::IoError(err)) if err.kind() == ErrorKind::UnexpectedEof => {
                return packet_count > 0;
            }
            Err(_) => return false,
        }
    }
}
