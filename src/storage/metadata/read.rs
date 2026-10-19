use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    path::Path,
};

use id3::{Tag, TagLike};
use yt_audio_downloader::{extract_video_id, is_youtube_url, youtube_id_to_i64};

use super::id3::{get_txxx, get_txxx_with_legacy};
use crate::{
    constants::{
        LEGACY_SC_ARTWORK_URL, LEGACY_SC_IDENTIFIER, LEGACY_SC_SOURCE_URL, TAG_ARTWORK_URL,
        TAG_PLATFORM, TAG_SOURCE_URL, TAG_TRACK_ID,
    },
    types::{SourcePlatform, TrackMetadata},
    utils::filename,
};

fn extract_track_id(tag: Option<&Tag>, source_url: Option<&str>, path: &Path) -> i64 {
    if let Some(id) = tag
        .and_then(|t| get_txxx_with_legacy(t, TAG_TRACK_ID, LEGACY_SC_IDENTIFIER))
        .and_then(|s| s.parse::<i64>().ok())
    {
        return id;
    }

    if let Some(url) = source_url
        && is_youtube_url(url)
        && let Ok(video_id) = extract_video_id(url)
    {
        return youtube_id_to_i64(&video_id);
    }

    let mut hasher = DefaultHasher::new();
    path.to_string_lossy().hash(&mut hasher);
    let hash = hasher.finish();

    let safe_53_bit = (hash & 0x001F_FFFF_FFFF_FFFF) as i64;
    if safe_53_bit == 0 { -1 } else { -safe_53_bit }
}

/// What: Extracts artist, title, source URL, and custom metadata from an audio file on disk.
/// Flow: Called by `MusicStorage::scan_library` for new or modified audio files.
/// Why:  Inspects ID3v2 tags (including custom TXXX user frames for track ID, source URL, artwork URL),
///       falling back to filename heuristics and stable hash generation for non-tagged local files.
pub fn extract_track_metadata(path: impl AsRef<Path>) -> Option<TrackMetadata> {
    let path = path.as_ref();
    let tag = Tag::read_from_path(path).ok();

    let artwork_url = tag
        .as_ref()
        .and_then(|t| get_txxx_with_legacy(t, TAG_ARTWORK_URL, LEGACY_SC_ARTWORK_URL));
    let source_url = tag
        .as_ref()
        .and_then(|t| get_txxx_with_legacy(t, TAG_SOURCE_URL, LEGACY_SC_SOURCE_URL));

    let platform = tag
        .as_ref()
        .and_then(|t| get_txxx(t, TAG_PLATFORM))
        .unwrap_or_else(|| {
            SourcePlatform::detect_from_optional_url(source_url.as_deref())
                .as_str()
                .to_string()
        });

    let id = extract_track_id(tag.as_ref(), source_url.as_deref(), path);

    let tag_artist = tag
        .as_ref()
        .and_then(|t| t.artist())
        .map(str::trim)
        .filter(|s| !s.is_empty());
    let tag_title = tag
        .as_ref()
        .and_then(|t| t.title())
        .map(str::trim)
        .filter(|s| !s.is_empty());

    let (artist, title) = match (tag_artist, tag_title) {
        (Some(a), Some(t)) => (a.to_string(), t.to_string()),
        (Some(a), None) => {
            let stem = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Unknown");
            (a.to_string(), stem.to_string())
        }
        (None, Some(t)) => ("Unknown".to_string(), t.to_string()),
        (None, None) => {
            let stem = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Unknown");
            filename::parse_track_metadata(stem, "Unknown")
        }
    };

    Some(TrackMetadata {
        id,
        artist,
        title,
        artwork_url,
        source_url,
        platform,
    })
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use id3::Version;

    use super::{super::id3::set_txxx, *};

    #[test]
    fn test_extract_track_metadata_without_tags() {
        let path1 = PathBuf::from("/var/lib/vortex-dl/Beauty in the Pain - Rebouz (192k).mp3");
        let path2 = PathBuf::from("/var/lib/vortex-dl/Beauty in the Pain - Rebouz (192k).mp3");
        let path3 = PathBuf::from("/var/lib/vortex-dl/Other Track.flac");

        let meta1 = extract_track_metadata(&path1).unwrap();
        let meta2 = extract_track_metadata(&path2).unwrap();
        let meta3 = extract_track_metadata(&path3).unwrap();

        assert_eq!(
            meta1.id, meta2.id,
            "Same path should yield same deterministic ID"
        );
        assert_ne!(
            meta1.id, meta3.id,
            "Different paths should yield different IDs"
        );
        assert!(
            meta1.id < 0,
            "Generated ID for local tracks should be negative"
        );
        assert_eq!(meta1.artist, "Beauty in the Pain");
        assert_eq!(meta1.title, "Rebouz (192k)");
    }

    #[test]
    fn test_youtube_source_url_id_extraction() {
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join("test_yt_extract_id.mp3");
        let _ = std::fs::File::create(&file_path);

        let mut tag = Tag::new();
        set_txxx(
            &mut tag,
            LEGACY_SC_SOURCE_URL,
            "https://www.youtube.com/watch?v=dQw4w9WgXcQ",
        );
        let _ = tag.write_to_path(&file_path, Version::Id3v23);

        let meta = extract_track_metadata(&file_path).unwrap();
        let expected_id = youtube_id_to_i64("dQw4w9WgXcQ");
        assert_eq!(meta.id, expected_id);

        let _ = std::fs::remove_file(&file_path);
    }
}
