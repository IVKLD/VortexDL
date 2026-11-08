use std::io;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum AudioVerifyError {
    #[error("I/O error during audio verification: {0}")]
    Io(#[from] io::Error),

    #[error("Audio file too small: {actual} bytes (minimum {min} bytes)")]
    TooSmall { actual: u64, min: u64 },

    #[error("Integrity check failed: expected {expected} bytes, got on disk {actual}")]
    SizeMismatch { expected: u64, actual: u64 },

    #[error("Audio integrity check failed: file is corrupted or truncated")]
    CorruptedOrTruncated,
}
