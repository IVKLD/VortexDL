use crate::error::{Result, YoutubeAudioError};
use regex::Regex;

pub fn extract_video_id(url_or_id: &str) -> Result<String> {
    let trimmed = url_or_id.trim();

    if trimmed.len() == 11 && trimmed.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
        return Ok(trimmed.to_string());
    }

    let re = Regex::new(
        r"(?:youtube\.com/(?:[^/]+/.+/|(?:v|e(?:mbed)?|shorts)/|.*[?&]v=)|youtu\.be/)([^?&/]{11})",
    )
    .map_err(|e| YoutubeAudioError::InvalidUrl(e.to_string()))?;

    re.captures(trimmed)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().to_string())
        .ok_or_else(|| YoutubeAudioError::VideoIdNotFound(url_or_id.to_string()))
}
