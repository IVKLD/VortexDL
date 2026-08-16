use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    path::Path,
};

use anyhow::Result;
use id3::{
    Content, Error as Id3Error, ErrorKind as Id3ErrorKind, Tag, TagLike, Version,
    frame::{ExtendedText, Frame, Picture, PictureType},
};
use url::Url;

use crate::{
    constants::{
        LEGACY_SC_ARTWORK_URL, LEGACY_SC_IDENTIFIER, LEGACY_SC_SOURCE_URL, TAG_ARTWORK_URL,
        TAG_PLATFORM, TAG_SOURCE_URL, TAG_TRACK_ID,
    },
    types::TrackMetadata,
    utils::filename,
};

fn get_tag(path: impl AsRef<Path>) -> Result<Tag> {
    let path = path.as_ref();
    match Tag::read_from_path(path) {
        Ok(tag) => Ok(tag),
        Err(Id3Error {
            kind: Id3ErrorKind::NoTag,
            ..
        }) => Ok(Tag::new()),
        Err(e) => {
            tracing::warn!(
                path = %path.display(),
                error = %e,
                "Failed to read ID3 tag, starting fresh",
            );
            Ok(Tag::new())
        }
    }
}

fn set_txxx(tag: &mut Tag, key: &str, value: &str) {
    tag.remove_extended_text(Some(key), None);
    tag.add_frame(Frame::with_content(
        "TXXX",
        Content::ExtendedText(ExtendedText {
            description: key.to_string(),
            value: value.to_string(),
        }),
    ));
}

fn get_txxx(tag: &Tag, key: &str) -> Option<String> {
    tag.extended_texts()
        .find(|f| f.description == key)
        .map(|f| f.value.clone())
}

fn get_txxx_with_legacy(tag: &Tag, key: &str, legacy_key: &str) -> Option<String> {
    get_txxx(tag, key).or_else(|| get_txxx(tag, legacy_key))
}

pub fn detect_platform_str(source_url: Option<&str>) -> &'static str {
    source_url
        .map(|u| {
            if yt_downloader_rs::is_youtube_url(u) {
                "youtube"
            } else {
                "soundcloud"
            }
        })
        .unwrap_or("local")
}

pub struct SaveTrackArgs<'a> {
    pub path: &'a Path,
    pub sc_id: &'a str,
    pub title: &'a str,
    pub artist: &'a str,
    pub artwork_url: Option<&'a Url>,
    pub source_url: Option<&'a Url>,
    pub artwork_data: Option<Vec<u8>>,
}

pub fn save_track_info(args: SaveTrackArgs) -> Result<()> {
    let mut tag = get_tag(args.path)?;

    tag.set_title(args.title);
    tag.set_artist(args.artist);

    let platform = detect_platform_str(args.source_url.map(|u| u.as_str()));

    set_txxx(&mut tag, TAG_TRACK_ID, args.sc_id);
    set_txxx(&mut tag, TAG_PLATFORM, platform);

    if let Some(url) = args.artwork_url {
        set_txxx(&mut tag, TAG_ARTWORK_URL, url.as_str());
    }
    if let Some(url) = args.source_url {
        set_txxx(&mut tag, TAG_SOURCE_URL, url.as_str());
    }

    set_txxx(&mut tag, LEGACY_SC_IDENTIFIER, args.sc_id);
    if let Some(url) = args.artwork_url {
        set_txxx(&mut tag, LEGACY_SC_ARTWORK_URL, url.as_str());
    }
    if let Some(url) = args.source_url {
        set_txxx(&mut tag, LEGACY_SC_SOURCE_URL, url.as_str());
    }

    if let Some(data) = args.artwork_data {
        tag.remove_all_pictures();
        tag.add_frame(Frame::with_content(
            "APIC",
            Content::Picture(Picture {
                mime_type: "image/jpeg".to_string(),
                picture_type: PictureType::CoverFront,
                description: "Cover".to_string(),
                data,
            }),
        ));
    }

    tag.write_to_path(args.path, Version::Id3v23)?;
    Ok(())
}

fn extract_track_id(tag: Option<&Tag>, source_url: Option<&str>, path: &Path) -> i64 {
    if let Some(id) = tag
        .and_then(|t| get_txxx_with_legacy(t, TAG_TRACK_ID, LEGACY_SC_IDENTIFIER))
        .and_then(|s| s.parse::<i64>().ok())
    {
        return id;
    }

    if let Some(url) = source_url
        && yt_downloader_rs::is_youtube_url(url)
        && let Ok(video_id) = yt_downloader_rs::extract_video_id(url)
    {
        return yt_downloader_rs::youtube_id_to_i64(&video_id);
    }

    let mut hasher = DefaultHasher::new();
    path.to_string_lossy().hash(&mut hasher);
    let hash = hasher.finish();

    let safe_53_bit = (hash & 0x001F_FFFF_FFFF_FFFF) as i64;
    if safe_53_bit == 0 { -1 } else { -safe_53_bit }
}

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
        .unwrap_or_else(|| detect_platform_str(source_url.as_deref()).to_string());

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

    use super::*;

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
        let expected_id = yt_downloader_rs::youtube_id_to_i64("dQw4w9WgXcQ");
        assert_eq!(meta.id, expected_id);

        let _ = std::fs::remove_file(&file_path);
    }
}
