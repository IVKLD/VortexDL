use std::path::Path;

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum AudioFormat {
    Mp3,
    Flac,
    Wav,
    M4a,
    Aac,
    Ogg,
    Opus,
    Wma,
    Alac,
    Aiff,
    Unknown,
}

impl AudioFormat {
    pub fn from_path(path: impl AsRef<Path>) -> Self {
        path.as_ref()
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| match ext.to_lowercase().as_str() {
                "mp3" => Self::Mp3,
                "flac" => Self::Flac,
                "wav" => Self::Wav,
                "m4a" => Self::M4a,
                "aac" => Self::Aac,
                "ogg" => Self::Ogg,
                "opus" => Self::Opus,
                "wma" => Self::Wma,
                "alac" => Self::Alac,
                "aiff" | "aif" => Self::Aiff,
                _ => Self::Unknown,
            })
            .unwrap_or(Self::Unknown)
    }
}
